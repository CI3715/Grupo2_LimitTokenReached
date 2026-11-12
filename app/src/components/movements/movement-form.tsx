"use client";

import { useId, useRef, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, type SubmitHandler } from "react-hook-form";
import type { Account, Currency } from "@/types/accounts";
import {
  expenseExceedsBalance,
  type CategoriaDTO,
  type MovimientoDTO,
  type MovementFormError,
  type RegistrarMovimientoRequest,
  type TipoRegistroMovimiento,
} from "@/types/movements";
import type { CrearCategoriaRequest } from "@/types/categories";
import { clearAmountInput } from "@/types/account-money";
import { registerMovementSchema, type RegisterMovementFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";
import { createCategorySchema, type CreateCategoryFormValues } from "@/types/schemas";
import "./movement-form.css";

const CREATE_CATEGORY_VALUE = "__crear_categoria__";

export interface MovementFormProps {
  account: Pick<Account, "id" | "nombre" | "moneda_codigo" | "saldo">;
  currency: Currency;
  categories: CategoriaDTO[];
  categoriesLoading?: boolean;
  categoriesError?: string;
  onRetryCategories?: () => void;
  onCreateCategory: (request: CrearCategoriaRequest) => Promise<CategoriaDTO>;
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
  return (
    whole.replace(/\B(?=(\d{3})+(?!\d))/g, ".") +
    (fraction ? `,${fraction}` : "")
  );
}

export function MovementForm({
  account,
  currency,
  categories,
  categoriesLoading = false,
  categoriesError,
  onRetryCategories,
  onCreateCategory,
  onRegister,
  onRegistered,
  onCancel,
}: MovementFormProps) {
  const prefix = useId();
  const inFlight = useRef(false);
  const [tipo, setTipo] = useState<TipoRegistroMovimiento>("Gasto");
  const [categoria, setCategoria] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<MovementFormError | null>(null);
  const [saved, setSaved] = useState<MovimientoDTO | null>(null);
  const [refreshFailed, setRefreshFailed] = useState(false);
  const [newCategoryName, setNewCategoryName] = useState("");
  const [newCategoryType, setNewCategoryType] =
    useState<TipoRegistroMovimiento>("Gasto");
  const [creatingCategory, setCreatingCategory] = useState(false);
  const [categoryCreationError, setCategoryCreationError] = useState<string | null>(null);
  const [confirmation, setConfirmation] =
    useState<RegistrarMovimientoRequest | null>(null);
  const form = useForm<RegisterMovementFormValues>({
    resolver: zodResolver(registerMovementSchema),
    defaultValues: {
      cuenta_id: account.id,
      tipo: "Gasto",
      monto: "",
      fecha: today(),
      categoria_id: "",
      nota: "",
      confirmar_saldo_negativo: false,
    },
  });
  const categoryForm = useForm<CreateCategoryFormValues>({
    resolver: zodResolver(createCategorySchema),
    defaultValues: { nombre: "", tipo: "Gasto" },
  });

  const available = categories.filter((category) => category.tipo === tipo);
  const hasCategory = available.some((category) => category.id === categoria);
  const locked = pending || !!saved || error?.verificarRegistro === true;
  const errorField =
    error?.campo === "categoria" || error?.campo === "categoría"
      ? "categoria_id"
      : error?.campo;

  function fieldError(name: keyof RegisterMovementFormValues | string) {
    return form.formState.errors[name as keyof RegisterMovementFormValues]?.message
      ? String(form.formState.errors[name as keyof RegisterMovementFormValues]?.message)
      : errorField === name
        ? error?.mensaje
        : undefined;
  }

  async function handleCreateCategory() {
    if (creatingCategory || locked) return;
    setCreatingCategory(true);
    setCategoryCreationError(null);
    try {
      const category = await onCreateCategory(await categoryForm.getValues());
      setCategoria(category.id);
      setTipo(category.tipo as TipoRegistroMovimiento);
      form.setValue("tipo", category.tipo as TipoRegistroMovimiento);
      form.setValue("categoria_id", category.id);
      categoryForm.reset({ nombre: "", tipo: category.tipo as "Ingreso" | "Gasto" });
    } catch (caught: unknown) {
      const failure = caught as { mensaje?: unknown };
      setCategoryCreationError(
        typeof failure.mensaje === "string"
          ? failure.mensaje
          : "No se pudo crear la categoría.",
      );
    } finally {
      setCreatingCategory(false);
    }
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
        mensaje:
          typeof failure?.mensaje === "string"
            ? failure.mensaje
            : "No se pudo registrar el movimiento.",
        campo: failure?.campo,
        verificarRegistro: failure?.verificarRegistro,
      };

      setError(nextError);
      applyBackendValidationError(caught, form.setError);
      const name =
        nextError.campo === "categoria" || nextError.campo === "categoría"
          ? "categoria_id"
          : nextError.campo;

      setPending(false);
      inFlight.current = false;
      requestAnimationFrame(() => {
        if (name) document.getElementById(`${prefix}-${name}`)?.focus();
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

  const submit: SubmitHandler<RegisterMovementFormValues> = (values) => {
    if (
      locked ||
      confirmation
    ) {
      return;
    }

    const request: RegistrarMovimientoRequest = {
      ...values,
      cuenta_id: account.id,
      tipo,
      monto: values.monto.trim().replace(",", "."),
      categoria_id: categoria,
      nota: values.nota?.trim() || null,
    };

    setError(null);
    if (tipo === "Gasto" && expenseExceedsBalance(request.monto, account.saldo)) {
      setConfirmation(request);
      return;
    }

    void persist(request);
  };

  return (
    <section className="movement-form-panel" aria-labelledby={`${prefix}-title`}>
      <header>
        <p className="eyebrow">Nuevo movimiento</p>
        <h2 id={`${prefix}-title`}>Registrar movimiento</h2>
        <p>
          {account.nombre} · {account.moneda_codigo}
        </p>
        <p>
          Saldo actual: {currency.simbolo} {displayAmount(account.saldo)}
        </p>
      </header>

      {/* React Hook Form creates the submit handler during render. */}
      {/* eslint-disable-next-line react-hooks/refs */}
      <form onSubmit={form.handleSubmit(submit)} aria-busy={pending}>
        {error && !errorField && (
          <p className="movement-error" role="alert">
            {error.mensaje}
          </p>
        )}
        {saved && (
          <p className="movement-success" role="status">
            Movimiento registrado correctamente.
          </p>
        )}
        {refreshFailed && (
          <p role="alert">
            El movimiento se guardó, pero no se pudo actualizar la vista. Cierra
            y vuelve a abrir el detalle de la cuenta.
          </p>
        )}

        <fieldset disabled={locked || !!confirmation}>
          <legend className="movement-sr-only">Datos del movimiento</legend>
          <label htmlFor={`${prefix}-tipo`}>Tipo</label>
          <select
            id={`${prefix}-tipo`}
            value={tipo}
            aria-invalid={!!fieldError("tipo")}
            aria-describedby={
              fieldError("tipo") ? `${prefix}-tipo-error` : undefined
            }
            {...form.register("tipo", {
              onChange: (event) => {
                setTipo(event.target.value as TipoRegistroMovimiento);
                setCategoria("");
                form.setValue("categoria_id", "");
                setError(null);
              },
            })}
          >
            <option value="Gasto">Gasto</option>
            <option value="Ingreso">Ingreso</option>
          </select>
          {fieldError("tipo") && (
            <small id={`${prefix}-tipo-error`} className="movement-error">
              {fieldError("tipo")}
            </small>
          )}

          <div className="movement-form-columns">
            <div>
              <label htmlFor={`${prefix}-monto`}>
                Monto ({account.moneda_codigo})
              </label>
              <input
                id={`${prefix}-monto`}
                type="text"
                inputMode="decimal"
                autoComplete="off"
                placeholder="Ej. 50,00"
                aria-invalid={!!fieldError("monto")}
                aria-describedby={`${prefix}-monto-help ${prefix}-monto-error`}
                onInput={(event) => {
                  event.currentTarget.value = clearAmountInput(
                    event.currentTarget.value,
                  );
                }}
                {...form.register("monto")}
              />
              <small id={`${prefix}-monto-error`} className="movement-error">
                {fieldError("monto")}
              </small>
            </div>

            <div>
              <label htmlFor={`${prefix}-fecha`}>Fecha</label>
              <input
                id={`${prefix}-fecha`}
                type="date"
                defaultValue={today()}
                aria-invalid={!!fieldError("fecha")}
                aria-describedby={`${prefix}-fecha-error`}
                {...form.register("fecha")}
              />
              <small id={`${prefix}-fecha-error`} className="movement-error">
                {fieldError("fecha")}
              </small>
            </div>
          </div>

          <label htmlFor={`${prefix}-categoria`}>Categoría</label>
          <select
            id={`${prefix}-categoria`}
            value={
              categoria === CREATE_CATEGORY_VALUE || hasCategory ? categoria : ""
            }
            disabled={
              categoriesLoading ||
              !!categoriesError ||
              available.length === 0
            }
            aria-invalid={!!fieldError("categoria_id")}
            aria-describedby={`${prefix}-categoria-help ${prefix}-categoria-error`}
            {...form.register("categoria_id", {
              onChange: (event) => setCategoria(event.target.value),
            })}
          >
            <option value="">Selecciona una categoría</option>
            <option value={CREATE_CATEGORY_VALUE}>+ Crear nueva categoría</option>
            {available.map((category) => (
              <option key={category.id} value={category.id}>
                {category.nombre}
              </option>
            ))}
          </select>
          <small id={`${prefix}-categoria-help`} role="status">
            {categoriesLoading
              ? "Cargando categorías…"
              : categoriesError
                ? categoriesError
                : available.length === 0
                  ? `No hay categorías de ${tipo.toLowerCase()} disponibles.`
                  : "Solo se muestran las categorías del tipo seleccionado."}
          </small>
          {categoriesError && onRetryCategories && (
            <button type="button" onClick={onRetryCategories}>
              Reintentar carga de categorías
            </button>
          )}
          <small id={`${prefix}-categoria-error`} className="movement-error">
            {fieldError("categoria_id")}
          </small>
          {categoria === CREATE_CATEGORY_VALUE && (
            <div className="movement-category-create">
              <div>
                <label htmlFor={`${prefix}-nueva-categoria`}>
                  Nombre de la nueva categoría
                </label>
                <input
                  id={`${prefix}-nueva-categoria`}
                  value={newCategoryName}
                  onChange={(event) => {
                    setNewCategoryName(event.target.value);
                    categoryForm.setValue("nombre", event.target.value);
                  }}
                  disabled={creatingCategory}
                  maxLength={80}
                />
                <label htmlFor={`${prefix}-tipo-nueva-categoria`}>Tipo</label>
                <select
                  id={`${prefix}-tipo-nueva-categoria`}
                  value={newCategoryType}
                  onChange={(event) => {
                    const value = event.target.value as TipoRegistroMovimiento;
                    setNewCategoryType(value);
                    categoryForm.setValue("tipo", value);
                  }}
                  disabled={creatingCategory}
                >
                  <option value="Gasto">Gasto</option>
                  <option value="Ingreso">Ingreso</option>
                </select>
                {categoryCreationError && (
                  <small className="movement-error" role="alert">
                    {categoryCreationError}
                  </small>
                )}
                <button
                  type="button"
                  className="account-secondary"
                  disabled={creatingCategory}
                  onClick={() => void categoryForm.handleSubmit(handleCreateCategory)()}
                >
                  {creatingCategory ? "Guardando…" : "Guardar categoría"}
                </button>
              </div>
            </div>
          )}

          <label htmlFor={`${prefix}-nota`}>Nota (opcional)</label>
          <textarea
            id={`${prefix}-nota`}
            rows={3}
            placeholder="Ej. Mercado del sábado"
            aria-invalid={!!fieldError("nota")}
            aria-describedby={`${prefix}-nota-error`}
            {...form.register("nota")}
          />
          <small id={`${prefix}-nota-error`} className="movement-error">
            {fieldError("nota")}
          </small>
        </fieldset>

        {confirmation && (
          <div className="movement-warning" role="alert">
            <p>
              Este gasto de {currency.simbolo}{" "}
              {displayAmount(confirmation.monto)} dejará la cuenta con saldo
              negativo. ¿Quieres registrarlo?
            </p>
            <button
              type="button"
              disabled={pending}
              onClick={() => setConfirmation(null)}
            >
              Volver al formulario
            </button>
            <button
              type="button"
              className="account-primary"
              disabled={pending}
              onClick={() =>
                void persist({
                  ...confirmation,
                  confirmar_saldo_negativo: true,
                })
              }
            >
              Confirmar gasto
            </button>
          </div>
        )}

        <footer>
          <button
            type="button"
            className="account-secondary"
            disabled={pending}
            onClick={onCancel}
          >
            {saved ? "Cerrar" : "Cancelar"}
          </button>
          {!saved && !confirmation && (
            <button
              type="submit"
              className="account-primary"
              disabled={
                locked ||
                categoriesLoading ||
                !!categoriesError ||
                !hasCategory ||
                currency.codigo !== account.moneda_codigo
              }
            >
              {pending ? "Registrando…" : "Registrar movimiento"}
            </button>
          )}
        </footer>
      </form>
    </section>
  );
}
