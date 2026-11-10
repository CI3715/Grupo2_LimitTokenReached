import { z } from "zod";
import { formFields } from "./fields";

export const registerRateSchema = z.object({
  moneda_origen: formFields.moneda_origen,
  moneda_destino: formFields.moneda_destino,
  valor: formFields.valor,
  fecha: formFields.fecha,
  fuente: formFields.fuente,
});

export type RegisterRateFormValues = z.infer<typeof registerRateSchema>;
