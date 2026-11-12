use tauri::AppHandle;
#[cfg(target_os = "android")]
use tauri::Manager;

const SERVICIO_KEYRING: &str = "cuentas-claras";
const CUENTA_KEYRING: &str = "sesion";

#[cfg(not(target_os = "android"))]
pub fn guardar_token(_: &AppHandle, token: &str) -> Result<(), String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, CUENTA_KEYRING)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    entrada
        .set_password(token)
        .map_err(|_| "No se pudo guardar la sesión de forma segura.".to_string())
}

#[cfg(target_os = "android")]
pub fn guardar_token(app: &AppHandle, token: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let ruta = ruta_token(app)?;
    let directorio = ruta
        .parent()
        .ok_or_else(|| "No se pudo determinar la carpeta de sesión.".to_string())?;
    std::fs::create_dir_all(directorio)
        .map_err(|_| "No se pudo crear el almacén local de sesión.".to_string())?;
    std::fs::write(&ruta, token).map_err(|_| "No se pudo guardar la sesión local.".to_string())?;
    std::fs::set_permissions(&ruta, std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "No se pudieron proteger los permisos de la sesión.".to_string())
}

#[cfg(not(target_os = "android"))]
pub fn leer_token(_: &AppHandle) -> Result<Option<String>, String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, CUENTA_KEYRING)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    match entrada.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("No se pudo leer la sesión del almacén seguro.".to_string()),
    }
}

#[cfg(target_os = "android")]
pub fn leer_token(app: &AppHandle) -> Result<Option<String>, String> {
    match std::fs::read_to_string(ruta_token(app)?) {
        Ok(token) => Ok(Some(token)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("No se pudo leer la sesión local.".to_string()),
    }
}

#[cfg(not(target_os = "android"))]
pub fn borrar_token(_: &AppHandle) -> Result<(), String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, CUENTA_KEYRING)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    match entrada.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("No se pudo borrar la sesión del almacén seguro.".to_string()),
    }
}

#[cfg(target_os = "android")]
pub fn borrar_token(app: &AppHandle) -> Result<(), String> {
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

pub fn obtener_token(app: &AppHandle) -> Result<String, String> {
    leer_token(app)?.ok_or_else(|| "No hay una sesión iniciada.".to_string())
}
