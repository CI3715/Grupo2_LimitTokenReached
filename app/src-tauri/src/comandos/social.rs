use contrato::{
    CrearAmistadRequestDTO, CrearGrupoRequestDTO, DetalleGrupoDTO, GastoCreadoDTO, GrupoDTO,
    InvitacionGrupoCreadaDTO, InvitarMiembroGrupoRequestDTO, ListaAmistadesDTO,
    RegistrarGastoRequestDTO, RespuestaInvitacionDTO, RespuestaInvitacionGrupoDTO,
    UsuarioResumenDTO,
};
use infraestructura::{ClienteSocialHttp, SocialError};
use tauri::AppHandle;
#[cfg(target_os = "android")]
use tauri::Manager;

const SERVICIO_KEYRING: &str = "cuentas-claras";
const CUENTA_KEYRING: &str = "sesion";

fn cliente_social() -> ClienteSocialHttp {
    let url = option_env!("SERVER_HOST_IP").unwrap_or("http://localhost:8000");
    ClienteSocialHttp::new(url)
}

#[cfg(not(target_os = "android"))]
fn leer_token(_: &AppHandle) -> Result<Option<String>, String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, CUENTA_KEYRING)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    match entrada.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("No se pudo leer la sesión del almacén seguro.".to_string()),
    }
}

#[cfg(target_os = "android")]
fn leer_token(app: &AppHandle) -> Result<Option<String>, String> {
    match std::fs::read_to_string(ruta_token(app)?) {
        Ok(token) => Ok(Some(token)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("No se pudo leer la sesión local.".to_string()),
    }
}

#[cfg(not(target_os = "android"))]
fn borrar_token(_: &AppHandle) -> Result<(), String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, CUENTA_KEYRING)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    match entrada.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("No se pudo borrar la sesión del almacén seguro.".to_string()),
    }
}

#[cfg(target_os = "android")]
fn borrar_token(app: &AppHandle) -> Result<(), String> {
    match std::fs::remove_file(ruta_token(app)?) {
        Ok(()) | Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("No se pudo borrar la sesión local.".to_string()),
    }
}

#[cfg(target_os = "android")]
fn ruta_token(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directorio| directorio.join("session.token"))
        .map_err(|_| "No se pudo obtener la carpeta privada de la aplicación.".to_string())
}

fn obtener_token(app: &AppHandle) -> Result<String, String> {
    leer_token(app)?.ok_or_else(|| "No hay una sesión iniciada.".to_string())
}

fn mensaje_error(app: &AppHandle, error: SocialError) -> String {
    if error.status().is_some_and(|status| status.as_u16() == 401) {
        if let Err(error_borrado) = borrar_token(app) {
            return error_borrado;
        }
        return "Tu sesión venció, inicia sesión de nuevo.".to_string();
    }
    error.mensaje_ui().to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn buscar_usuario(app: AppHandle, usuario: String) -> Result<UsuarioResumenDTO, String> {
    if usuario.is_empty() {
        return Err("El nombre de usuario es obligatorio.".to_string());
    }
    let token = obtener_token(&app)?;
    cliente_social()
        .buscar_usuario(&usuario, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn invitar_amistad(
    app: AppHandle,
    datos: CrearAmistadRequestDTO,
) -> Result<contrato::AmistadCreadaDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .invitar_amistad(&datos, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn responder_invitacion_amistad(
    app: AppHandle,
    invitacion_id: String,
    aceptar: bool,
) -> Result<RespuestaInvitacionDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .responder_invitacion_amistad(&invitacion_id, aceptar, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn listar_amistades(app: AppHandle) -> Result<ListaAmistadesDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .listar_amistades(&token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn eliminar_amistad(app: AppHandle, amistad_id: String) -> Result<(), String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .eliminar_amistad(&amistad_id, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn crear_grupo(app: AppHandle, datos: CrearGrupoRequestDTO) -> Result<GrupoDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .crear_grupo(&datos, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn invitar_miembro_grupo(
    app: AppHandle,
    grupo_id: String,
    datos: InvitarMiembroGrupoRequestDTO,
) -> Result<InvitacionGrupoCreadaDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .invitar_miembro_grupo(&grupo_id, &datos, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn responder_invitacion_grupo(
    app: AppHandle,
    grupo_id: String,
    aceptar: bool,
) -> Result<RespuestaInvitacionGrupoDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .responder_invitacion_grupo(&grupo_id, aceptar, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn registrar_gasto(
    app: AppHandle,
    grupo_id: String,
    datos: RegistrarGastoRequestDTO,
) -> Result<GastoCreadoDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .registrar_gasto(&grupo_id, &datos, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn obtener_detalle_grupo(
    app: AppHandle,
    grupo_id: String,
) -> Result<DetalleGrupoDTO, String> {
    let token = obtener_token(&app)?;
    cliente_social()
        .obtener_detalle_grupo(&grupo_id, &token)
        .await
        .map_err(|error| mensaje_error(&app, error))
}
