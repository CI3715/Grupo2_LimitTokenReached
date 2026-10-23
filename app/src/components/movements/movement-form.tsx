"use client";

import { useId, useRef, useState, type FormEvent } from "react";
import type { Account, Currency } from "@/types/accounts";
import {
  expenseExceedsBalance,
  type CategoriaDTO,
  type MovimientoDTO,
  type MovementFormError,
  type RegistrarMovimientoRequest,
  type TipoRegistroMovimiento,
} from "@/types/movements";
import "./movement-form.css";

export interface MovementFormProps {
  account: Pick<Account, "id" | "nombre" | "moneda_codigo" | "saldo">;
  currency: Currency;
  categories: CategoriaDTO[];
  categoriesLoading?: boolean;
  categoriesError?: string;
  onRetryCategories?: () => void;
  onRegister: (request: RegistrarMovimientoRequest) => Promise<MovimientoDTO>;
  onRegistered: (movement: MovimientoDTO) => void | Promise<void>;
  onCancel: () => void;
}

function today(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
}

function displayAmount(amount: string): string {
  const [whole, fraction] = amount.split(".");
  return whole.replace(/\B(?=(\d{3})+(?!\d))/g, ".") + (fraction ? `,${fraction}` : "");
}

export function MovementForm({
  account, currency, categories, categoriesLoading = false, categoriesError,
  onRetryCategories, onRegister, onRegistered, onCancel,
}: MovementFormProps) {
  const prefix = useId();
  const form = useRef<HTMLFormElement>(null);
  const inFlight = useRef(false);
  const [tipo, setTipo] = useState<TipoRegistroMovimiento>("Gasto");
  const [categoria, setCategoria] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<MovementFormError | null>(null);
  const [saved, setSaved] = useState<MovimientoDTO | null>(null);
  const [refreshFailed, setRefreshFailed] = useState(false);
  const [confirmation, setConfirmation] = useState<RegistrarMovimientoRequest | null>(null);
  const available = categories.filter((category) => category.tipo === tipo);
  const hasCategory = available.some((category) => category.id === categoria);
  const locked = pending || !!saved || error?.verificarRegistro === true;
  const errorField = error?.campo === "categoria" || error?.campo === "categoría"
    ? "categoria_id" : error?.campo;

  function fieldError(name: string) {
    return errorField === name ? error?.mensaje : undefined;
  }

  async function persist(request: RegistrarMovimientoRequest) {
    if (inFlight.current || locked) return;
    inFlight.current = true;
    setPending(true);
    setError(null);
    setConfirmation(null);
    let movement: MovimientoDTO;
    try {
      movement = await onRegister(request);
    } catch (caught: unknown) {
      const failure = caught as Partial<MovementFormError> | null;
      const nextError: MovementFormError = {
        mensaje: typeof failure?.mensaje === "string" ? failure.mensaje : "No se pudo registrar el movimiento.",
        campo: failure?.campo,
        verificarRegistro: failure?.verificarRegistro,
      };
      setError(nextError);
      const name = nextError.campo === "categoria" || nextError.campo === "categoría"
        ? "categoria_id" : nextError.campo;
      setPending(false);
      inFlight.current = false;
      requestAnimationFrame(() => {
        const field = name ? form.current?.elements.namedItem(name) : null;
        if (field instanceof HTMLElement) field.focus();
      });
      return;
    }

    // Desde aquí el movimiento ya se guardó: fallar al refrescar no permite reenviarlo.
    setSaved(movement);
    try {
      await onRegistered(movement);
    } catch {
      setRefreshFailed(true);
    } finally {
      setPending(false);
      inFlight.current = false;
    }
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (locked || confirmation || !hasCategory || categoriesLoading || categoriesError) return;
    const data = new FormData(event.currentTarget);
    const request: RegistrarMovimientoRequest = {
      cuenta_id: account.id,
      tipo,
      monto: String(data.get("monto") ?? "").trim().replace(",", "."),
      fecha: String(data.get("fecha") ?? ""),
      categoria_id: categoria,
      nota: String(data.get("nota") ?? "").trim() || null,
    };
    setError(null);
    if (tipo === "Gasto" && expenseExceedsBalance(request.monto, account.saldo)) {
      setConfirmation(request);
      return;
    }
    void persist(request);
  }

  return (
    <section className="movement-form-panel" aria-labelledby={`${prefix}-title`}>
      <header>
        <h2 id={`${prefix}-title`}>Registrar movimiento</h2>
        <p>{account.nombre} · {account.moneda_codigo}</p>
        <p>Saldo actual: {currency.simbolo} {displayAmount(account.saldo)}</p>
      </header>
      <form ref={form} onSubmit={submit} aria-busy={pending}>
        {error && <p className="movement-error" role="alert">{error.mensaje}</p>}
        {saved && <p className="movement-success" role="status">Movimiento registrado correctamente.</p>}
        {refreshFailed && <p role="alert">El movimiento se guardó, pero no se pudo actualizar la vista. Cierra y vuelve a abrir el detalle de la cuenta.</p>}
        <fieldset disabled={locked || !!confirmation}>
          <legend className="movement-sr-only">Datos del movimiento</legend>
          <label htmlFor={`${prefix}-tipo`}>Tipo</label>
          <select id={`${prefix}-tipo`} name="tipo" value={tipo}
            aria-invalid={!!fieldError("tipo")} aria-describedby={fieldError("tipo") ? `${prefix}-tipo-error` : undefined}
            onChange={(event) => {
              setTipo(event.target.value as TipoRegistroMovimiento);
              setCategoria("");
              setError(null);
            }}>
            <option value="Gasto">Gasto</option>
            <option value="Ingreso">Ingreso</option>
          </select>
          {fieldError("tipo") && <small id={`${prefix}-tipo-error`} className="movement-error">{fieldError("tipo")}</small>}
          <div className="movement-form-columns">
            <div>
              <label htmlFor={`${prefix}-monto`}>Monto ({account.moneda_codigo})</label>
              <input id={`${prefix}-monto`} name="monto" type="text" inputMode="decimal"
                autoComplete="off" required pattern="[0-9]+([.,][0-9]+)?" placeholder="Ej. 50,00"
                aria-invalid={!!fieldError("monto")} aria-describedby={`${prefix}-monto-help ${prefix}-monto-error`} />
              <small id={`${prefix}-monto-help`}>Cantidad positiva, hasta {currency.decimales} decimales. Sin separadores de miles.</small>
              <small id={`${prefix}-monto-error`} className="movement-error">{fieldError("monto")}</small>
            </div>
            <div>
              <label htmlFor={`${prefix}-fecha`}>Fecha</label>
              <input id={`${prefix}-fecha`} name="fecha" type="date" required defaultValue={today()} max={today()}
                aria-invalid={!!fieldError("fecha")} aria-describedby={`${prefix}-fecha-error`} />
              <small id={`${prefix}-fecha-error`} className="movement-error">{fieldError("fecha")}</small>
            </div>
          </div>
          <label htmlFor={`${prefix}-categoria`}>Categoría</label>
          <select id={`${prefix}-categoria`} name="categoria_id" required value={hasCategory ? categoria : ""}
            disabled={categoriesLoading || !!categoriesError || available.length === 0}
            aria-invalid={!!fieldError("categoria_id")} aria-describedby={`${prefix}-categoria-help ${prefix}-categoria-error`}
            onChange={(event) => setCategoria(event.target.value)}>
            <option value="">Selecciona una categoría</option>
            {available.map((category) => <option key={category.id} value={category.id}>{category.nombre}</option>)}
          </select>
          <small id={`${prefix}-categoria-help`} role="status">
            {categoriesLoading ? "Cargando categorías…" : categoriesError
              ? categoriesError : available.length === 0
                ? `No hay categorías de ${tipo.toLowerCase()} disponibles.` : "Solo se muestran las categorías del tipo seleccionado."}
          </small>
          {categoriesError && onRetryCategories && <button type="button" onClick={onRetryCategories}>Reintentar carga de categorías</button>}
          <small id={`${prefix}-categoria-error`} className="movement-error">{fieldError("categoria_id")}</small>
          <label htmlFor={`${prefix}-nota`}>Nota (opcional)</label>
          <textarea id={`${prefix}-nota`} name="nota" rows={3} placeholder="Ej. Mercado del sábado"
            aria-invalid={!!fieldError("nota")} aria-describedby={`${prefix}-nota-error`} />
          <small id={`${prefix}-nota-error`} className="movement-error">{fieldError("nota")}</small>
        </fieldset>
        {confirmation && <div className="movement-warning" role="alert">
          <p>Este gasto de {currency.simbolo} {displayAmount(confirmation.monto)} dejará la cuenta con saldo negativo. ¿Quieres registrarlo?</p>
          <button type="button" disabled={pending} onClick={() => setConfirmation(null)}>Volver al formulario</button>
          <button type="button" className="account-primary" disabled={pending} onClick={() => void persist(confirmation)}>Confirmar gasto</button>
        </div>}
        <footer>
          <button type="button" className="account-secondary" disabled={pending} onClick={onCancel}>{saved ? "Cerrar" : "Cancelar"}</button>
          {!saved && !confirmation && <button type="submit" className="account-primary"
            disabled={locked || categoriesLoading || !!categoriesError || !hasCategory || currency.codigo !== account.moneda_codigo}>
            {pending ? "Registrando…" : "Registrar movimiento"}
          </button>}
        </footer>
      </form>
    </section>
  );
}
