import { z } from "zod";
import { formFields } from "./fields";

export const createCurrencySchema = z.object({
  codigo: formFields.codigo,
  nombre: formFields.nombre,
  simbolo: formFields.simbolo,
  decimales: formFields.decimales,
});

export type CreateCurrencyFormValues = z.infer<typeof createCurrencySchema>;
