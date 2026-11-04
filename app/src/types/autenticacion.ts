// Espejo del contrato E5 de Rust y del servidor Python.
// Las reglas de registro y la verificación de credenciales corresponden al servidor.
export interface RegistrarUsuarioRequest {
  nombre_usuario: string;
  nombre_visible: string;
  correo: string;
  contrasena: string;
}

export interface IniciarSesionRequest {
  correo: string;
  contrasena: string;
}

export interface UsuarioDTO {
  id: string;
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
