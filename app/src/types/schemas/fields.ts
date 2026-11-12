import { z } from "zod";

export const formFields = {
  correo: z.string().trim(),
  contrasena: z.string(),
  nombre_usuario: z.string().trim(),
  nombre_visible: z.string().trim(),
  codigo: z.string().trim().toUpperCase(),
  simbolo: z.string().trim().min(1),
  decimales: z.coerce.number().int(),
  fecha: z.string().date(),
  fuente: z.string().trim(),
  valor: z.string().regex(/^\d+(?:[.,]\d{1,8})?$/),
  monto: z.string(),
  nombre: z.string().trim(),
  moneda_codigo: z.string().trim(),
  moneda_origen: z.string().trim(),
  moneda_destino: z.string().trim(),
  categoria_id: z.string().trim(),
  movimiento_id: z.string().trim(),
  nota: z.string().trim().nullable(),
} as const;
