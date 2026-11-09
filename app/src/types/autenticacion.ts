// Espejo del contrato E5 de Rust y del servidor Python.
// Las reglas de registro y la verificación de credenciales corresponden al servidor.
import type { LoginFormValues, RegisterFormValues } from "./schemas";

export type RegistrarUsuarioRequest = RegisterFormValues;
export type IniciarSesionRequest = LoginFormValues;

export interface UsuarioDTO {
  nombre_usuario: string;
  nombre_visible: string;
  correo: string;
  fecha_alta: string;
}

export interface SesionDTO {
  token: string;
  vence_en: string;
}

export type { ErrorValidacionDTO as ErrorAutenticacionDTO } from "./error_validacion";
