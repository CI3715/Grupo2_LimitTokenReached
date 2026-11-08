import { invoke } from "@tauri-apps/api/core";

import type {
  IniciarSesionRequest,
  RegistrarUsuarioRequest,
  UsuarioDTO,
} from "@/types/autenticacion";

export function registrarUsuario(datos: RegistrarUsuarioRequest) {
  return invoke<UsuarioDTO>("registrar_usuario", { ...datos });
}

export function iniciarSesion(datos: IniciarSesionRequest) {
  return invoke<UsuarioDTO>("iniciar_sesion", { ...datos });
}

export function obtenerPerfil() {
  return invoke<UsuarioDTO>("obtener_perfil");
}

export function cerrarSesion() {
  return invoke<void>("cerrar_sesion");
}
