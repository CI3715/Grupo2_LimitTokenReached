"use client";

import { useState } from "react";
import type { Currency } from "@/types/accounts";
import { GlobalForm, FormField } from "@/components/forms";
import { registerRateSchema, type RegisterRateFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";
import type { UseFormReturn } from "react-hook-form";

type Props = {
  currencies: Currency[];
  initialOrigin?: string;
  initialDestination?: string;
  initialDate: string;
  onRegister: (request: RegisterRateFormValues) => Promise<void>;
};

function today() {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

export function RegistrarTasaForm({
  currencies,
  initialOrigin,
  initialDestination,
  initialDate,
  onRegister,
}: Props) {
  const [origin, setOrigin] = useState("");
  const [destination, setDestination] = useState("");
  const [saving, setSaving] = useState(false);

  const selectedOrigin = currencies.some(
    (currency) => currency.codigo === origin,
  )
    ? origin
    : currencies.some((currency) => currency.codigo === initialOrigin)
      ? initialOrigin!
      : currencies[0]?.codigo ?? "";
  const selectedDestination =
    currencies.some(
      (currency) =>
        currency.codigo === destination &&
        currency.codigo !== selectedOrigin,
    )
      ? destination
      : currencies.some(
            (currency) =>
              currency.codigo === initialDestination &&
              currency.codigo !== selectedOrigin,
          )
        ? initialDestination!
        : currencies.find(
            (currency) => currency.codigo !== selectedOrigin,
          )?.codigo ?? "";

  async function submit(values: RegisterRateFormValues, form: UseFormReturn<RegisterRateFormValues>) {

    if (selectedOrigin === selectedDestination) {
      form.setError("moneda_destino", {
        type: "validate",
        message: "Selecciona dos monedas diferentes.",
      });
      return;
    }

    setSaving(true);

    try {
      await onRegister({ ...values, moneda_origen: selectedOrigin, moneda_destino: selectedDestination });
    } catch (cause) {
      if (!applyBackendValidationError(cause, form.setError)) {
        form.setError("root.server", {
          type: "server",
          message: cause instanceof Error ? cause.message : "No se pudo registrar la tasa.",
        });
      }
    } finally {
      setSaving(false);
    }
  }

  return (
    <section
      className="patrimonio-rate-form"
      id="registrar-tasa-form"
      aria-labelledby="registrar-tasa-title"
    >
      <h2 id="registrar-tasa-title">Registrar tasa de cambio</h2>

      <GlobalForm<RegisterRateFormValues>
        schema={registerRateSchema}
        defaultValues={{
          moneda_origen: selectedOrigin,
          moneda_destino: selectedDestination,
          valor: "",
          fecha: initialDate,
          fuente: "",
        }}
        onSubmit={submit}
      >
        {(form) => (
          <>
            <FormField<RegisterRateFormValues> name="moneda_origen" label="Moneda de origen">
              {(props) => (
                <select {...props} value={selectedOrigin} onChange={(event) => setOrigin(event.currentTarget.value)} disabled={saving || currencies.length === 0}>
                  {currencies.map((currency) => <option key={currency.codigo} value={currency.codigo}>{currency.codigo} · {currency.nombre}</option>)}
                </select>
              )}
            </FormField>
            <FormField<RegisterRateFormValues> name="moneda_destino" label="Moneda de destino">
              {(props) => (
                <select {...props} value={selectedDestination} onChange={(event) => setDestination(event.currentTarget.value)} disabled={saving || currencies.length < 2}>
                  {currencies.map((currency) => <option key={currency.codigo} value={currency.codigo} disabled={currency.codigo === selectedOrigin}>{currency.codigo} · {currency.nombre}</option>)}
                </select>
              )}
            </FormField>
            <FormField<RegisterRateFormValues> name="valor" label="Valor (unidades de destino por una de origen)">
              {(props) => <input {...props} type="text" inputMode="decimal" placeholder="Ej. 887.00" disabled={saving} {...form.register("valor")} />}
            </FormField>
            <FormField<RegisterRateFormValues> name="fecha" label="Fecha">
              {(props) => <input {...props} type="date" max={today()} disabled={saving} {...form.register("fecha")} />}
            </FormField>
            <FormField<RegisterRateFormValues> name="fuente" label="Fuente">
              {(props) => <input {...props} type="text" placeholder="Ej. BCV" disabled={saving} {...form.register("fuente")} />}
            </FormField>
            <button type="submit" disabled={saving || currencies.length < 2}>
              {saving ? "Registrando…" : "Registrar tasa"}
            </button>
          </>
        )}
      </GlobalForm>
    </section>
  );
}