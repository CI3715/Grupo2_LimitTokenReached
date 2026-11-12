use crate::puertos::ClienteSocial;
use contrato::{
    AmistadCreadaDTO, CrearAmistadRequestDTO, CrearGrupoRequestDTO, DetalleGrupoDTO,
    GastoCreadoDTO, GrupoDTO, InvitacionGrupoCreadaDTO, InvitarMiembroGrupoRequestDTO,
    ListaAmistadesDTO, RegistrarGastoRequestDTO, RespuestaInvitacionDTO,
    RespuestaInvitacionGrupoDTO, UsuarioResumenDTO,
};

pub async fn buscar_usuario(
    cliente: &dyn ClienteSocial,
    usuario: String,
    token: String,
) -> Result<UsuarioResumenDTO, crate::puertos::ErrorOperacion> {
    cliente.buscar_usuario(usuario, token).await
}

pub async fn invitar_amistad(
    cliente: &dyn ClienteSocial,
    datos: CrearAmistadRequestDTO,
    token: String,
) -> Result<AmistadCreadaDTO, crate::puertos::ErrorOperacion> {
    cliente.invitar_amistad(datos, token).await
}

pub async fn responder_invitacion_amistad(
    cliente: &dyn ClienteSocial,
    invitacion_id: String,
    aceptar: bool,
    token: String,
) -> Result<RespuestaInvitacionDTO, crate::puertos::ErrorOperacion> {
    cliente
        .responder_invitacion_amistad(invitacion_id, aceptar, token)
        .await
}

pub async fn listar_amistades(
    cliente: &dyn ClienteSocial,
    token: String,
) -> Result<ListaAmistadesDTO, crate::puertos::ErrorOperacion> {
    cliente.listar_amistades(token).await
}

pub async fn eliminar_amistad(
    cliente: &dyn ClienteSocial,
    amistad_id: String,
    token: String,
) -> Result<(), crate::puertos::ErrorOperacion> {
    cliente.eliminar_amistad(amistad_id, token).await
}

pub async fn eliminar_amistad_por_usuario(
    cliente: &dyn ClienteSocial,
    nombre_usuario: String,
    token: String,
) -> Result<(), crate::puertos::ErrorOperacion> {
    cliente
        .eliminar_amistad_por_usuario(nombre_usuario, token)
        .await
}

pub async fn crear_grupo(
    cliente: &dyn ClienteSocial,
    datos: CrearGrupoRequestDTO,
    token: String,
) -> Result<GrupoDTO, crate::puertos::ErrorOperacion> {
    cliente.crear_grupo(datos, token).await
}

pub async fn invitar_miembro_grupo(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    datos: InvitarMiembroGrupoRequestDTO,
    token: String,
) -> Result<InvitacionGrupoCreadaDTO, crate::puertos::ErrorOperacion> {
    cliente.invitar_miembro_grupo(grupo_id, datos, token).await
}

pub async fn responder_invitacion_grupo(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    aceptar: bool,
    token: String,
) -> Result<RespuestaInvitacionGrupoDTO, crate::puertos::ErrorOperacion> {
    cliente
        .responder_invitacion_grupo(grupo_id, aceptar, token)
        .await
}

pub async fn registrar_gasto(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    datos: RegistrarGastoRequestDTO,
    token: String,
) -> Result<GastoCreadoDTO, crate::puertos::ErrorOperacion> {
    cliente.registrar_gasto(grupo_id, datos, token).await
}

pub async fn obtener_detalle_grupo(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    token: String,
) -> Result<DetalleGrupoDTO, crate::puertos::ErrorOperacion> {
    cliente.obtener_detalle_grupo(grupo_id, token).await
}
