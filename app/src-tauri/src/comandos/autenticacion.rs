use super::sesion::{borrar_token, guardar_token, leer_token};
use infraestructura::{
    ClienteAutenticacionHttp, InicioSesionHttp, RegistroUsuarioHttp, UsuarioHttp,
};
use tauri::AppHandle;

fn cliente_auth() -> ClienteAutenticacionHttp {
    let url = option_env!("SERVER_HOST_IP").unwrap_or("http://localhost:8000");
    ClienteAutenticacionHttp::new(url)
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
