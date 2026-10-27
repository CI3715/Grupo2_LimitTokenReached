"use client";
import type { ErrorValidacionDTO } from "@/types/error_validacion";
import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  accountTypeLabels,
  type AccountType,
  type CreateAccountInput,
  type Currency,
  type CurrencyCode,
} from "@/types/accounts";
import { sanitizeFreeAmountInput } from "@/types/account-money";

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
  const [currencyCode, setCurrencyCode] = useState<CurrencyCode>(
    currencies[0]?.codigo ?? "USD",
  );
  const selectedCurrency = currencies.find(
    (currency) => currency.codigo === currencyCode,
  );
  const decimalPlaces = selectedCurrency?.decimales ?? 0;

  function fieldError(...names: string[]) {
    return names.includes(validationError?.campo ?? "")
      ? validationError?.mensaje
      : undefined;
  }
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
        {validationError && !validationError.campo && (
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
              aria-invalid={!!fieldError("nombre")}
              aria-describedby={fieldError("nombre") ? "nombre-error" : undefined}
            />
            {fieldError("nombre") && (
              <small id="nombre-error" className="account-error" role="alert">
                {fieldError("nombre")}
              </small>
            )}
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
            {fieldError("tipo") && (
              <small id="tipo-error" className="account-error" role="alert">
                {fieldError("tipo")}
              </small>
            )}
          </fieldset>
          <div className="account-form-columns">
            <label>
              Moneda
              <select
                name="moneda_codigo"
                value={currencyCode}
                onChange={(event) =>
                  setCurrencyCode(event.currentTarget.value as CurrencyCode)
                }
                aria-invalid={!!fieldError("moneda", "moneda_codigo")}
                aria-describedby={
                  fieldError("moneda", "moneda_codigo")
                    ? "moneda-error"
                    : undefined
                }
              >
                {currencies.map((currency) => (
                  <option key={currency.codigo} value={currency.codigo}>
                    {currency.codigo} · {currency.nombre}
                  </option>
                ))}
              </select>
              {fieldError("moneda", "moneda_codigo") && (
                <small id="moneda-error" className="account-error" role="alert">
                  {fieldError("moneda", "moneda_codigo")}
                </small>
              )}
            </label>
            <label>
              Saldo inicial
              <input
                name="saldo_inicial"
                inputMode="decimal"
                defaultValue="0"
                type="text"
                aria-invalid={!!fieldError("saldo", "saldo_inicial")}
                aria-describedby={`saldo-inicial-help${
                  fieldError("saldo", "saldo_inicial")
                    ? " saldo-inicial-error"
                    : ""
                }`}
                required
                onInput={(e) => {
                  e.currentTarget.value = sanitizeFreeAmountInput(
                    e.currentTarget.value,
                  );
                }}
              />
              <small id="saldo-inicial-help">
                Hasta {decimalPlaces} {decimalPlaces === 1 ? "decimal" : "decimales"}; puede ser 0.
              </small>
              {fieldError("saldo", "saldo_inicial") && (
                <small id="saldo-inicial-error" className="account-error" role="alert">
                  {fieldError("saldo", "saldo_inicial")}
                </small>
              )}
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