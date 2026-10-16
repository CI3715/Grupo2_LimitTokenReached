"use client";
import type { ErrorValidacionDTO } from "@/app/types/error_validacion";
import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  accountTypeLabels,
  type AccountType,
  type CreateAccountInput,
  type Currency,
  type CurrencyCode,
} from "@/types/accounts";

type Props = {
   currencies: Currency[];
  onClose: () => void;
  onCreate: (input: CreateAccountInput) => Promise<void>;
  validationError: ErrorValidacionDTO | null;
};
export function NewAccountModal({
  currencies,
  onClose,
  onCreate,
  validationError,
}: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  
  const [type, setType] = useState<AccountType>("efectivo");
  useEffect(() => {
    const node = dialog.current;
    const previousFocus = document.activeElement as HTMLElement | null;
    node?.showModal();
    node?.querySelector<HTMLInputElement>('[name="nombre"]')?.focus();
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      node?.close();
      document.body.style.overflow = previousOverflow;
      previousFocus?.focus();
    };
  }, []);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);

    // Obtener input del formulario
    const form = event.currentTarget;
    const inputNombre = form.elements.namedItem("nombre") as HTMLInputElement;
    const nombreValue = inputNombre.value;

    // Verificación del nombre para que no sean espacios únicamente
    if (nombreValue.trim().length === 0 && nombreValue.length > 0) {
      inputNombre.setCustomValidity("El nombre no puede ser únicamente espacios.");
      inputNombre.reportValidity(); // Fuerza al navegador a mostrar la burbuja de inmediato
      return;
    } else {
      // Limpia cualquier error previo si el texto ahora es válido
      inputNombre.setCustomValidity("");
    }

    try {
  await onCreate({
    nombre: String(data.get("nombre") ?? ""),
    tipo: type,
    moneda_codigo: String(data.get("moneda_codigo")) as CurrencyCode,
    saldo_inicial: String(data.get("saldo_inicial") ?? "").replace(",", "."),
  });

  onClose();
} catch {
  // El error ya fue guardado y mostrado por page.tsx.
  // El modal debe permanecer abierto.
}
  }
  return (
    <dialog
      ref={dialog}
      className="new-account-dialog"
      aria-labelledby="new-account-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          const bounds = event.currentTarget.getBoundingClientRect();
          if (
            event.clientX < bounds.left ||
            event.clientX > bounds.right ||
            event.clientY < bounds.top ||
            event.clientY > bounds.bottom
          )
            onClose();
        }
      }}
    >
      <header>
        <div>
          <p className="eyebrow">Nueva cuenta</p>
          <h2 id="new-account-title">Agrega una cuenta</h2>
          <p>Completa los datos de tu cuenta para organizar tus saldos.</p>
        </div>
        <button
          className="close-account-dialog"
          aria-label="Cerrar formulario"
          onClick={onClose}
        >
          ×
        </button>
      </header>
      <form onSubmit={submit}>
        {validationError && (
          <p className="account-error" role="alert">
            {validationError.mensaje}
        </p>
)}
        <fieldset>
          <label>
            Nombre
            <input
              autoFocus
              name="nombre"
              placeholder="Ej. Cuenta nómina"
              autoComplete="off"
              required
              onInput={(e) => e.currentTarget.setCustomValidity("")}
            />
          </label>
          <fieldset className="account-type-options">
            <legend>Tipo de cuenta</legend>
            {(Object.keys(accountTypeLabels) as AccountType[]).map((value) => (
              <label key={value} className={type === value ? "selected" : ""}>
                <input
                  type="radio"
                  name="tipo"
                  value={value}
                  checked={type === value}
                  onChange={() => setType(value)}
                />
                {accountTypeLabels[value]}
              </label>
            ))}
          </fieldset>
          <div className="account-form-columns">
            <label>
              Moneda
              <select name="moneda_codigo">
                {currencies.map((currency) => (
                  <option key={currency.codigo} value={currency.codigo}>
                    {currency.codigo} · {currency.nombre}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Saldo inicial
              <input
                name="saldo_inicial"
                inputMode="decimal"
                defaultValue="0"
                type="number"
                min="0"
                step="0.01"
                required
              />
              <small>Hasta 2 decimales; puede ser 0.</small>
            </label>
          </div>
        </fieldset>
        <footer>
          <button type="button" className="account-secondary" onClick={onClose}>
            Cancelar
          </button>
          <button className="account-primary" type="submit">
            Crear cuenta
          </button>
        </footer>
      </form>
    </dialog>
  );
}
