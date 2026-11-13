import type { UseFormReturn } from "react-hook-form";

import { createCurrency } from "@/services/createCurrency";
import type { Currency } from "@/types/accounts";
import type { CreateCurrencyFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";

export async function crearMonedaDesdeFormulario(
  values: CreateCurrencyFormValues,
  form: UseFormReturn<CreateCurrencyFormValues>,
  mostrarError: (mensaje: string) => void,
  alCrear: (currency: Currency) => void,
) {
  try {
    const currency = await createCurrency(values);
    alCrear(currency);
    form.reset();
  } catch (cause: unknown) {
    console.error("No se pudo crear la moneda.", cause);
    if (!applyBackendValidationError(cause, form.setError)) {
      mostrarError("No se pudo crear la moneda.");
    }
  }
}
