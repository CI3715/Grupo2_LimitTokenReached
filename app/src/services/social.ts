import { invoke } from "@tauri-apps/api/core";

import type {
  AmistadCreadaDTO,
  CrearGrupoRequestDTO,
  CrearAmistadRequestDTO,
  DetalleGrupoDTO,
  GastoCreadoDTO,
  GrupoDTO,
  InvitacionGrupoCreadaDTO,
  InvitarMiembroGrupoRequestDTO,
  ListaAmistadesDTO,
  ListaGruposDTO,
  ParteGastoDTO,
  RegistrarGastoRequestDTO,
  RespuestaInvitacionGrupoDTO,
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

export function listarGrupos() {
  return invoke<ListaGruposDTO>("listar_grupos");
}

export function crearGrupo(datos: CrearGrupoRequestDTO) {
  return invoke<GrupoDTO>("crear_grupo", { datos });
}

export function invitarMiembroGrupo(
  grupoId: string,
  datos: InvitarMiembroGrupoRequestDTO,
) {
  return invoke<InvitacionGrupoCreadaDTO>("invitar_miembro_grupo", {
    grupo_id: grupoId,
    datos,
  });
}

export function responderInvitacionGrupo(grupoId: string, aceptar: boolean) {
  return invoke<RespuestaInvitacionGrupoDTO>("responder_invitacion_grupo", {
    grupo_id: grupoId,
    aceptar,
  });
}

export function registrarGastoGrupo(
  grupoId: string,
  datos: RegistrarGastoRequestDTO,
) {
  return invoke<GastoCreadoDTO>("registrar_gasto", { grupo_id: grupoId, datos });
}

export function obtenerDetalleGrupo(grupoId: string) {
  return invoke<DetalleGrupoDTO>("obtener_detalle_grupo", {
    grupo_id: grupoId,
  });
}

export function previsualizarDivisionGrupo(datos: RegistrarGastoRequestDTO) {
  return invoke<ParteGastoDTO[]>("previsualizar_division", { datos });
}
