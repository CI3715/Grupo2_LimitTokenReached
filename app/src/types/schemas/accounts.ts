import { z } from "zod";
import { formFields } from "./fields";

export const accountTypeSchema = z.enum([
  "efectivo",
  "banco",
  "billetera_digital",
]);

export const createAccountSchema = z.object({
  nombre: formFields.nombre,
  tipo: accountTypeSchema,
  moneda_codigo: formFields.moneda_codigo,
  saldo_inicial: formFields.monto,
});

export type CreateAccountFormValues = z.infer<typeof createAccountSchema>;
