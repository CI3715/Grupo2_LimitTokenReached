import { z } from "zod";
import { formFields } from "./fields";

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

export type LoginFormValues = z.infer<typeof loginSchema>;
export type RegisterFormValues = z.infer<typeof registerSchema>;
