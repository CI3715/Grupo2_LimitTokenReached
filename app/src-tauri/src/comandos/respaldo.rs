use aplicacion::puertos::TransporteSincronizacion;
use contrato::comandos_sync::ConfigDestinoSyncDTO;
use infraestructura::{CredencialesGit, TransporteCarpeta, TransporteGit};
use std::hash::{Hash, Hasher};
use tauri::{AppHandle, Manager, State};

const SERVICIO_KEYRING: &str = "cuentas-claras-sincronizacion";
const CUENTA_TOKEN_GIT: &str = "git-token";

pub struct IdentificadorDispositivo(pub String);

fn ruta_configuracion(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directorio| directorio.join("sincronizacion.json"))
        .map_err(|_| "No se pudo acceder a la configuración local.".to_string())
}

fn remoto_contiene_credenciales(remoto: &str) -> bool {
    remoto
        .split_once("://")
        .and_then(|(_, resto)| resto.split('/').next())
        .is_some_and(|autoridad| autoridad.contains('@'))
}

fn leer_configuracion_local(app: &AppHandle) -> Result<Option<ConfigDestinoSyncDTO>, String> {
    let ruta = ruta_configuracion(app)?;
    let contenido = match std::fs::read(&ruta) {
        Ok(contenido) => contenido,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("No se pudo leer la configuración de sincronización.".to_string()),
    };
    let mut configuracion: ConfigDestinoSyncDTO = serde_json::from_slice(&contenido)
        .map_err(|_| "La configuración de sincronización está dañada.".to_string())?;
    configuracion.credenciales = None;
    Ok(Some(configuracion))
}

fn crear_transporte(
    app: &AppHandle,
    configuracion: &ConfigDestinoSyncDTO,
    dispositivo: &str,
) -> Result<Box<dyn TransporteSincronizacion>, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|_| {
            "No se pudo acceder al almacenamiento local de la aplicación.".to_string()
        })?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    configuracion.tipo.hash(&mut hasher);
    configuracion.ruta_o_url.hash(&mut hasher);
    let directorio_sync = app_data
        .join("sincronizacion")
        .join(format!("{:016x}", hasher.finish()));
    let estado_local = directorio_sync.join("lotes_leidos.json");
    match configuracion.tipo.as_str() {
        "carpeta" => Ok(Box::new(TransporteCarpeta::nuevo(
            &configuracion.ruta_o_url,
            estado_local,
            dispositivo,
        )?)),
        "git" => {
            let credenciales = obtener_token_git()?.map(|secreto| {
                let (usuario, secreto) = match secreto.split_once(':') {
                    Some((usuario, secreto)) => (usuario, secreto),
                    None => ("x-access-token", secreto.as_str()),
                };
                CredencialesGit {
                    usuario: usuario.to_string(),
                    secreto: secreto.to_string(),
                }
            });
            Ok(Box::new(TransporteGit::nuevo(
                &configuracion.ruta_o_url,
                directorio_sync.join("repositorio"),
                estado_local,
                "",
                dispositivo,
                credenciales,
            )?))
        }
        _ => Err("El tipo de destino de sincronización no es válido.".to_string()),
    }
}

fn guardar_configuracion_local(
    app: &AppHandle,
    configuracion: &ConfigDestinoSyncDTO,
) -> Result<(), String> {
    let ruta = ruta_configuracion(app)?;
    let directorio = ruta
        .parent()
        .ok_or_else(|| "No se pudo determinar la carpeta de configuración.".to_string())?;
    std::fs::create_dir_all(directorio)
        .map_err(|_| "No se pudo crear la carpeta de configuración.".to_string())?;

    let temporal = directorio.join(format!(".sincronizacion-{}.tmp", uuid::Uuid::new_v4()));
    let contenido = serde_json::to_vec(configuracion)
        .map_err(|_| "No se pudo serializar la configuración de sincronización.".to_string())?;
    let resultado = (|| {
        use std::io::Write;
        let mut archivo = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|_| "No se pudo guardar la configuración de sincronización.".to_string())?;
        archivo
            .write_all(&contenido)
            .map_err(|_| "No se pudo guardar la configuración de sincronización.".to_string())?;
        archivo
            .sync_all()
            .map_err(|_| "No se pudo guardar la configuración de sincronización.".to_string())?;
        std::fs::rename(&temporal, ruta)
            .map_err(|_| "No se pudo actualizar la configuración de sincronización.".to_string())
    })();
    if resultado.is_err() {
        let _ = std::fs::remove_file(temporal);
    }
    resultado
}

#[cfg(not(target_os = "android"))]
fn guardar_secreto(cuenta: &str, secreto: &str) -> Result<(), String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, cuenta)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    entrada
        .set_password(secreto)
        .map_err(|_| "No se pudo guardar el secreto de forma segura.".to_string())
}

#[cfg(target_os = "android")]
fn guardar_secreto(_: &str, _: &str) -> Result<(), String> {
    Err("El almacén seguro del sistema no está disponible en este dispositivo.".to_string())
}

#[cfg(not(target_os = "android"))]
fn leer_secreto(cuenta: &str) -> Result<Option<String>, String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, cuenta)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    match entrada.get_password() {
        Ok(secreto) => Ok(Some(secreto)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("No se pudo leer el secreto del almacén seguro.".to_string()),
    }
}

#[cfg(target_os = "android")]
fn leer_secreto(_: &str) -> Result<Option<String>, String> {
    Err("El almacén seguro del sistema no está disponible en este dispositivo.".to_string())
}

#[cfg(not(target_os = "android"))]
fn borrar_secreto(cuenta: &str) -> Result<(), String> {
    let entrada = keyring::Entry::new(SERVICIO_KEYRING, cuenta)
        .map_err(|_| "No se pudo acceder al almacén seguro del sistema.".to_string())?;
    match entrada.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("No se pudo borrar el secreto del almacén seguro.".to_string()),
    }
}

#[cfg(target_os = "android")]
fn borrar_secreto(_: &str) -> Result<(), String> {
    Err("El almacén seguro del sistema no está disponible en este dispositivo.".to_string())
}

pub fn obtener_token_git() -> Result<Option<String>, String> {
    leer_secreto(CUENTA_TOKEN_GIT)
}

pub fn obtener_frase_sincronizacion() -> Result<Option<String>, String> {
    leer_secreto("frase-sincronizacion")
}

#[tauri::command]
pub fn guardar_configuracion_sincronizacion(
    app: AppHandle,
    mut configuracion: ConfigDestinoSyncDTO,
) -> Result<(), String> {
    configuracion.tipo = configuracion.tipo.trim().to_ascii_lowercase();
    configuracion.ruta_o_url = configuracion.ruta_o_url.trim().to_string();
    if !matches!(configuracion.tipo.as_str(), "git" | "carpeta") {
        return Err("El destino debe ser de tipo 'git' o 'carpeta'.".to_string());
    }
    if configuracion.ruta_o_url.is_empty() {
        return Err("Indica la URL del repositorio o la ruta de la carpeta.".to_string());
    }
    if configuracion.tipo == "git" && remoto_contiene_credenciales(&configuracion.ruta_o_url) {
        return Err(
            "Guarda las credenciales de Git en el almacén seguro, no en la URL.".to_string(),
        );
    }

    let token_git = if configuracion.tipo == "git" {
        configuracion.credenciales.take()
    } else {
        if configuracion.credenciales.is_some() {
            return Err("Las credenciales solo se admiten para un destino Git.".to_string());
        }
        None
    };
    if let Some(token) = token_git.as_deref() {
        let credencial_incompleta = token
            .split_once(':')
            .is_some_and(|(usuario, secreto)| usuario.trim().is_empty() || secreto.trim().is_empty());
        if token.trim().is_empty() {
            return Err("El token de Git no puede estar vacío.".to_string());
        }
        if credencial_incompleta {
            return Err("Las credenciales de Git están incompletas.".to_string());
        }
        guardar_secreto(CUENTA_TOKEN_GIT, token.trim())?;
    } else {
        borrar_secreto(CUENTA_TOKEN_GIT)?;
    }

    configuracion.credenciales = None;
    guardar_configuracion_local(&app, &configuracion)
}

#[tauri::command]
pub fn obtener_configuracion_sincronizacion(
    app: AppHandle,
) -> Result<Option<ConfigDestinoSyncDTO>, String> {
    leer_configuracion_local(&app)
}

#[tauri::command]
pub fn contar_lotes_sincronizacion(
    app: AppHandle,
    dispositivo: State<'_, IdentificadorDispositivo>,
) -> Result<usize, String> {
    let configuracion = leer_configuracion_local(&app)?
        .ok_or_else(|| "Primero configura un destino de sincronización.".to_string())?;
    let transporte = crear_transporte(&app, &configuracion, &dispositivo.0)?;
    Ok(transporte.leer_lotes_nuevos()?.len())
}

#[tauri::command]
pub fn guardar_frase_sincronizacion(
    frase_de_paso: String,
    confirmacion: String,
) -> Result<(), String> {
    if frase_de_paso.chars().count() < 12 {
        return Err("La frase de paso debe tener al menos 12 caracteres.".to_string());
    }
    if frase_de_paso != confirmacion {
        return Err("Las frases de paso no coinciden.".to_string());
    }
    guardar_secreto("frase-sincronizacion", &frase_de_paso)
}
