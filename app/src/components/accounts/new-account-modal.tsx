"use client";
import { useEffect, useRef, useState } from "react";
import {
  accountTypeLabels,
  type AccountType,
  type Currency,
  type CurrencyCode,
} from "@/types/accounts";
import { clearAmountInput } from "@/types/account-money";
import { GlobalForm, FormField } from "@/components/forms";
import "./new-account-modal.css";
import { createAccountSchema, type CreateAccountFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";
import type { UseFormReturn } from "react-hook-form";

type Props = {
   currencies: Currency[];
  onClose: () => void;
  onCreate: (input: CreateAccountFormValues) => Promise<void>;
};
export function NewAccountModal({
  currencies,
  onClose,
  onCreate,

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

  async function submit(data: CreateAccountFormValues, form: UseFormReturn<CreateAccountFormValues>) {
    try {
      await onCreate(data);
      onClose();
    } catch (error: unknown) {
      if (!applyBackendValidationError(error, form.setError)) {
        form.setError("root.server", {
          type: "server",
          message: "No se pudo crear la cuenta.",
        });
      }
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
      </header>
      <GlobalForm<CreateAccountFormValues>
        schema={createAccountSchema}
        defaultValues={{
          nombre: "",
          tipo: "efectivo",
          moneda_codigo: currencyCode,
          saldo_inicial: "0",
        }}
        onSubmit={submit}
      >
        {(form) => (
        <>
        <fieldset>
          <FormField<CreateAccountFormValues> name="nombre" label="Nombre">
            {(props) => <input {...props} autoFocus placeholder="Ej. Cuenta nómina" autoComplete="off" {...form.register("nombre")} />}
          </FormField>
          <fieldset className="account-type-options">
            <legend>Tipo de cuenta</legend>
            {(Object.keys(accountTypeLabels) as AccountType[]).map((value) => (
              <label key={value} className={type === value ? "selected" : ""}>
                <input
                  type="radio"
                  value={value}
                  checked={type === value}
                  onChange={() => {
                    setType(value);
                    form.setValue("tipo", value, { shouldValidate: true });
                  }}
                />
                {accountTypeLabels[value]}
              </label>
            ))}
          </fieldset>
          <div className="account-form-columns">
            <label>
              Moneda
              <select
                value={currencyCode}
                {...form.register("moneda_codigo", {
                  onChange: (event) => {
                    setCurrencyCode(event.target.value as CurrencyCode);
                  },
                })}
              >
                {currencies.map((currency) => (
                  <option key={currency.codigo} value={currency.codigo}>
                    {currency.codigo} · {currency.nombre}
                  </option>
                ))}
              </select>
            </label>
            <FormField<CreateAccountFormValues> name="saldo_inicial" label="Saldo inicial" help={`Hasta ${decimalPlaces} ${decimalPlaces === 1 ? "decimal" : "decimales"}; puede ser 0.`}>
              {(props) => <input
                {...props}
                inputMode="decimal"
                defaultValue="0"
                type="text"
                onInput={(e) => {
                  e.currentTarget.value = clearAmountInput(
                    e.currentTarget.value,
                  );
                }}
                {...form.register("saldo_inicial")}
              />
              }
            </FormField>
          </div>
        </fieldset>
        <footer>
          <button type="button" className="account-secondary" onClick={onClose}>
            Cancelar
          </button>
          <button className="account-primary" type="submit" disabled={form.formState.isSubmitting}>
            Crear cuenta
          </button>
        </footer>
        </>
        )}
      </GlobalForm>
    </dialog>
  );
}