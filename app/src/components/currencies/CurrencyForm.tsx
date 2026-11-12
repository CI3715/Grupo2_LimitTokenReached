"use client";

import type { UseFormReturn } from "react-hook-form";

import { FormField, GlobalForm } from "@/components/forms";
import {
  createCurrencySchema,
  type CreateCurrencyFormValues,
} from "@/types/schemas";

type CurrencyFormProps = {
  onSubmit: (
    values: CreateCurrencyFormValues,
    form: UseFormReturn<CreateCurrencyFormValues>,
  ) => void | Promise<void>;
};

export function CurrencyForm({ onSubmit }: CurrencyFormProps) {
  return (
    <GlobalForm<CreateCurrencyFormValues>
      className="form-grid form-grid-two-columns"
      schema={createCurrencySchema}
      defaultValues={{ codigo: "", nombre: "", simbolo: "", decimales: 2 }}
      onSubmit={onSubmit}
    >
      {(form) => (
        <>
          <FormField<CreateCurrencyFormValues>
            name="codigo"
            label="Código"
            help="3 a 5 letras mayúsculas; debe ser único."
          >
            {(props) => (
              <input
                {...props}
                autoCapitalize="characters"
                autoComplete="off"
                {...form.register("codigo")}
              />
            )}
          </FormField>
          <FormField<CreateCurrencyFormValues> name="nombre" label="Nombre">
            {(props) => <input {...props} {...form.register("nombre")} />}
          </FormField>
          <FormField<CreateCurrencyFormValues> name="simbolo" label="Símbolo">
            {(props) => <input {...props} {...form.register("simbolo")} />}
          </FormField>
          <FormField<CreateCurrencyFormValues>
            name="decimales"
            label="Decimales"
          >
            {(props) => (
              <input
                {...props}
                type="number"
                {...form.register("decimales")}
              />
            )}
          </FormField>
          <button
            className="account-primary"
            type="submit"
            disabled={form.formState.isSubmitting}
          >
            {form.formState.isSubmitting ? "Guardando…" : "Crear moneda"}
          </button>
        </>
      )}
    </GlobalForm>
  );
}
