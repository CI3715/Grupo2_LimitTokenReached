use infraestructura::{
    ClienteAutenticacionHttp, InicioSesionHttp, RegistroUsuarioHttp, UsuarioHttp,
};
use tauri::AppHandle;
#[cfg(target_os = "android")]
use tauri::Manager;

const SERVICIO_KEYRING: &str = "cuentas-claras";
const CUENTA_KEYRING: &str = "sesion";

fn cliente_auth() -> ClienteAutenticacionHttp {
    let url = option_env!("SERVER_HOST_IP").unwrap_or("http://localhost:8000");
    ClienteAutenticacionHttp::new(url)
}

#[cfg(not(target_os = "android"))]
fn guardar_token(_: &AppHandle, token: &str) -> Result<(), String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, CUENTA_KEYRING)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    entrada
        .set_password(token)
        .map_err(|_| "No se pudo guardar la sesión de forma segura.".to_string())
}

#[cfg(target_os = "android")]
fn guardar_token(app: &AppHandle, token: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let ruta = ruta_token(app)?;
    let directorio = ruta
        .parent()
        .ok_or_else(|| "No se pudo determinar la carpeta de sesión.".to_string())?;
    std::fs::create_dir_all(directorio)
        .map_err(|_| "No se pudo crear el almacén local de sesión.".to_string())?;
    std::fs::write(&ruta, token)
        .map_err(|_| "No se pudo guardar la sesión local.".to_string())?;
    std::fs::set_permissions(&ruta, std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "No se pudieron proteger los permisos de la sesión.".to_string())
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
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
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

#[tauri::command(rename_all = "snake_case")]
pub async fn registrar_usuario(
    nombre_usuario: String,
    nombre_visible: String,
    correo: String,
    contrasena: String,
) -> Result<UsuarioHttp, String> {
    let cliente = cliente_auth();
    cliente
        .registrar_usuario(&RegistroUsuarioHttp {
            nombre_usuario,
            nombre_visible,
            correo,
            contrasena,
        })
        .await
        .map_err(|error| error.mensaje_ui())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn iniciar_sesion(
    app: AppHandle,
    correo: String,
    contrasena: String,
) -> Result<UsuarioHttp, String> {
    let cliente = cliente_auth();
    let sesion = cliente
        .iniciar_sesion(&InicioSesionHttp { correo, contrasena })
        .await
        .map_err(|error| error.mensaje_ui())?;

    guardar_token(&app, &sesion.token)?;
    match cliente.obtener_perfil(&sesion.token).await {
        Ok(usuario) => Ok(usuario),
        Err(error) if error.status().is_some_and(|s| s.as_u16() == 401) => {
            borrar_token(&app)?;
            Err("La sesión no pudo validarse. Inicia sesión de nuevo.".to_string())
        }
        Err(error) => Err(error.mensaje_ui()),
    }
}

#[tauri::command]
pub async fn obtener_perfil(app: AppHandle) -> Result<UsuarioHttp, String> {
    let Some(token) = leer_token(&app)? else {
        return Err("No hay una sesión iniciada.".to_string());
    };

    match cliente_auth().obtener_perfil(&token).await {
        Ok(usuario) => Ok(usuario),
        Err(error) if error.status().is_some_and(|s| s.as_u16() == 401) => {
            borrar_token(&app)?;
            Err("Tu sesión venció, inicia sesión de nuevo.".to_string())
        }
        Err(error) => Err(error.mensaje_ui()),
    }
}

#[tauri::command]
pub fn cerrar_sesion(app: AppHandle) -> Result<(), String> {
    borrar_token(&app)
}
