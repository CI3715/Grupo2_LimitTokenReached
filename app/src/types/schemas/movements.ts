import { z } from "zod";
import { formFields } from "./fields";

export const movementTypeSchema = z.enum(["Ingreso", "Gasto"]);

export const registerMovementSchema = z.object({
  cuenta_id: z.string().trim().min(1),
  tipo: movementTypeSchema,
  monto: formFields.monto,
  fecha: formFields.fecha,
  categoria_id: formFields.categoria_id,
  nota: formFields.nota,
  confirmar_saldo_negativo: z.boolean().optional(),
});

export const correctMovementSchema = z.object({
  movimiento_id: formFields.movimiento_id,
  monto_correcto: formFields.monto,
  confirmar_saldo_negativo: z.boolean().optional(),
});

export type RegisterMovementFormValues = z.infer<typeof registerMovementSchema>;
export type CorrectMovementFormValues = z.infer<typeof correctMovementSchema>;
