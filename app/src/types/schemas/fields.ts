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
