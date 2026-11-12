use super::sesion::{borrar_token, obtener_token};
use aplicacion::puertos::{ClienteSocial, ErrorOperacion};
use contrato::{
    CrearAmistadRequestDTO, CrearGrupoRequestDTO, DetalleGrupoDTO, GastoCreadoDTO, GrupoDTO,
    InvitacionGrupoCreadaDTO, InvitarMiembroGrupoRequestDTO, ListaAmistadesDTO,
    RegistrarGastoRequestDTO, RespuestaInvitacionDTO, RespuestaInvitacionGrupoDTO,
    UsuarioResumenDTO,
};
use tauri::{AppHandle, State};

fn mensaje_error(app: &AppHandle, error: ErrorOperacion) -> String {
    if error.status == Some(401) {
        if let Err(error_borrado) = borrar_token(app) {
            return error_borrado;
        }
        return "Tu sesión venció, inicia sesión de nuevo.".to_string();
    }
    error.mensaje
}

#[tauri::command(rename_all = "snake_case")]
pub async fn buscar_usuario(
    app: AppHandle,
    usuario: String,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<UsuarioResumenDTO, String> {
    if usuario.is_empty() {
        return Err("El nombre de usuario es obligatorio.".to_string());
    }
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::buscar_usuario(cliente.inner().as_ref(), usuario, token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn invitar_amistad(
    app: AppHandle,
    datos: CrearAmistadRequestDTO,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<contrato::AmistadCreadaDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::invitar_amistad(cliente.inner().as_ref(), datos, token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn responder_invitacion_amistad(
    app: AppHandle,
    invitacion_id: String,
    aceptar: bool,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<RespuestaInvitacionDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::responder_invitacion_amistad(
        cliente.inner().as_ref(),
        invitacion_id,
        aceptar,
        token,
    )
    .await
    .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn listar_amistades(
    app: AppHandle,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<ListaAmistadesDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::listar_amistades(cliente.inner().as_ref(), token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn eliminar_amistad(
    app: AppHandle,
    amistad_id: String,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<(), String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::eliminar_amistad(cliente.inner().as_ref(), amistad_id, token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn crear_grupo(
    app: AppHandle,
    datos: CrearGrupoRequestDTO,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<GrupoDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::crear_grupo(cliente.inner().as_ref(), datos, token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn invitar_miembro_grupo(
    app: AppHandle,
    grupo_id: String,
    datos: InvitarMiembroGrupoRequestDTO,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<InvitacionGrupoCreadaDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::invitar_miembro_grupo(
        cliente.inner().as_ref(),
        grupo_id,
        datos,
        token,
    )
    .await
    .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn responder_invitacion_grupo(
    app: AppHandle,
    grupo_id: String,
    aceptar: bool,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<RespuestaInvitacionGrupoDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::responder_invitacion_grupo(
        cliente.inner().as_ref(),
        grupo_id,
        aceptar,
        token,
    )
    .await
    .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn registrar_gasto(
    app: AppHandle,
    grupo_id: String,
    datos: RegistrarGastoRequestDTO,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<GastoCreadoDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::registrar_gasto(cliente.inner().as_ref(), grupo_id, datos, token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn obtener_detalle_grupo(
    app: AppHandle,
    grupo_id: String,
    cliente: State<'_, Box<dyn ClienteSocial>>,
) -> Result<DetalleGrupoDTO, String> {
    let token = obtener_token(&app)?;
    aplicacion::casos_uso::social::obtener_detalle_grupo(cliente.inner().as_ref(), grupo_id, token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}
