import { z } from "zod";

export const formFields = {
  correo: z.string().trim().email(),
  contrasena: z.string().min(10),
  nombre_usuario: z.string().trim().min(3).max(30),
  nombre_visible: z.string().trim().min(1),
  codigo: z.string().trim().toUpperCase().min(3).max(5),
  simbolo: z.string().trim().min(1),
  decimales: z.coerce.number().int().min(0).max(8),
  fecha: z.string().date(),
  fuente: z.string().trim().min(1),
  valor: z.string().regex(/^\d+(?:[.,]\d{1,8})?$/),
  monto: z.string().regex(/^\d+(?:[.,]\d+)?$/),
  nombre: z.string().trim().min(1),
  moneda_codigo: z.string().trim().min(1),
  moneda_origen: z.string().trim().min(1),
  moneda_destino: z.string().trim().min(1),
  categoria_id: z.string().trim().min(1),
  movimiento_id: z.string().trim().min(1),
  nota: z.string().trim().nullable(),
} as const;

export const accountTypeSchema = z.enum(["efectivo", "banco", "billetera_digital"]);
export const movementTypeSchema = z.enum(["Ingreso", "Gasto"]);

export const loginSchema = z.object({
  correo: formFields.correo,
  contrasena: formFields.contrasena,
});
export const registerSchema = z.object({
  nombre_usuario: formFields.nombre_usuario,
  nombre_visible: formFields.nombre_visible,
  correo: formFields.correo,
  contrasena: formFields.contrasena,
});
export const createAccountSchema = z.object({
  nombre: formFields.nombre,
  tipo: accountTypeSchema,
  moneda_codigo: formFields.moneda_codigo,
  saldo_inicial: formFields.monto,
});
export const createCurrencySchema = z.object({
  codigo: formFields.codigo,
  nombre: formFields.nombre,
  simbolo: formFields.simbolo,
  decimales: formFields.decimales,
});
export const registerRateSchema = z.object({
  moneda_origen: formFields.moneda_origen,
  moneda_destino: formFields.moneda_destino,
  valor: formFields.valor,
  fecha: formFields.fecha,
  fuente: formFields.fuente,
});
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
export const createCategorySchema = z.object({
  nombre: formFields.nombre,
  tipo: movementTypeSchema,
});

export type LoginFormValues = z.infer<typeof loginSchema>;
export type RegisterFormValues = z.infer<typeof registerSchema>;
export type CreateAccountFormValues = z.infer<typeof createAccountSchema>;
export type CreateCurrencyFormValues = z.infer<typeof createCurrencySchema>;
export type RegisterRateFormValues = z.infer<typeof registerRateSchema>;
export type RegisterMovementFormValues = z.infer<typeof registerMovementSchema>;
export type CorrectMovementFormValues = z.infer<typeof correctMovementSchema>;
export type CreateCategoryFormValues = z.infer<typeof createCategorySchema>;
