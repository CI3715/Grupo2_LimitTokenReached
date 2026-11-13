import { invoke } from "@tauri-apps/api/core";

import type {
  AmistadCreadaDTO,
  CrearAmistadRequestDTO,
  ListaAmistadesDTO,
  RespuestaInvitacionDTO,
  UsuarioResumenDTO,
} from "@/types/social";

export function buscarUsuario(usuario: string) {
  return invoke<UsuarioResumenDTO>("buscar_usuario", { usuario });
}

export function invitarAmistad(datos: CrearAmistadRequestDTO) {
  return invoke<AmistadCreadaDTO>("invitar_amistad", { datos });
}

export function responderInvitacionAmistad(
  invitacionId: string,
  aceptar: boolean,
) {
  return invoke<RespuestaInvitacionDTO>("responder_invitacion_amistad", {
    invitacion_id: invitacionId,
    aceptar,
  });
}

export function listarAmistades() {
  return invoke<ListaAmistadesDTO>("listar_amistades");
}

export function eliminarAmistad(nombreUsuario: string) {
  return invoke<void>("eliminar_amistad", { nombre_usuario: nombreUsuario });
}
