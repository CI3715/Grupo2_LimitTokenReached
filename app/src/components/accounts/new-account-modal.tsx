"use client";

import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  accountTypeLabels,
  type AccountError,
  type AccountType,
  type CreateAccountInput,
  type Currency,
  type CurrencyCode,
} from "@/types/accounts";

type Props = {
  currencies: Currency[];
  preview: boolean;
  onClose: () => void;
  onCreate: (input: CreateAccountInput) => Promise<AccountError | undefined>;
};
export function NewAccountModal({
  currencies,
  preview,
  onClose,
  onCreate,
}: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [type, setType] = useState<AccountType>("efectivo");
  const [error, setError] = useState<AccountError>();
  const [saving, setSaving] = useState(false);
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
    if (saving) return;
    const data = new FormData(event.currentTarget);
    setSaving(true);
    setError(undefined);
    const result = await onCreate({
      nombre: String(data.get("nombre") ?? ""),
      tipo: type,
      moneda: String(data.get("moneda")) as CurrencyCode,
      saldo_inicial: String(data.get("saldo_inicial") ?? "").replace(",", "."),
    });
    setSaving(false);
    if (result) {
      setError(result);
      if (result.campo)
        dialog.current
          ?.querySelector<HTMLElement>(`[name="${result.campo}"]`)
          ?.focus();
    } else onClose();
  }
  const errorProps = (field: string) => ({
    "aria-invalid": error?.campo === field || undefined,
    "aria-describedby":
      error?.campo === field ? "account-form-error" : undefined,
  });
  return (
    <dialog
      ref={dialog}
      className="new-account-dialog"
      aria-labelledby="new-account-title"
      onCancel={(event) => {
        event.preventDefault();
        if (!saving) onClose();
      }}
      onClick={(event) => {
        if (event.target === event.currentTarget && !saving) {
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
          <p>
            {preview
              ? "Prueba el formulario con datos de muestra."
              : "Se guardará únicamente en este dispositivo."}
          </p>
        </div>
        <button
          className="close-account-dialog"
          aria-label="Cerrar formulario"
          disabled={saving}
          onClick={onClose}
        >
          ×
        </button>
      </header>
      <form onSubmit={submit} noValidate>
        <fieldset disabled={saving}>
          <label>
            Nombre
            <input
              autoFocus
              name="nombre"
              placeholder="Ej. Cuenta nómina"
              autoComplete="off"
              {...errorProps("nombre")}
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
              <select name="moneda" {...errorProps("moneda")}>
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
                {...errorProps("saldo_inicial")}
              />
              <small>Hasta 2 decimales; puede ser 0.</small>
            </label>
          </div>
        </fieldset>
        {error && (
          <p id="account-form-error" className="account-error" role="alert">
            {error.mensaje}
          </p>
        )}
        <footer>
          <button
            type="button"
            className="account-secondary"
            disabled={saving}
            onClick={onClose}
          >
            Cancelar
          </button>
          <button className="account-primary" type="submit" disabled={saving}>
            {saving ? "Guardando…" : "Crear cuenta"}
          </button>
        </footer>
      </form>
    </dialog>
  );
}
