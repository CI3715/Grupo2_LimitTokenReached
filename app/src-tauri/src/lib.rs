
use aplicacion::casos_uso::verificar_conexion;
use infraestructura::ClienteSocialHttp;
use serde::Serialize;

/// Estructura DTO serializable a JSON hacia el Frontend (Next.js)
#[derive(Serialize)]
pub struct PingResultadoUI {
    pub estado: String,
    pub mensaje: String,
    pub version: String,
    pub timestamp: String,
    pub latencia_ms: u128,
}

#[tauri::command]
async fn ping_servidor() -> Result<PingResultadoUI, String> {
    let url_servidor = option_env!("SERVER_HOST_IP").unwrap_or("http://localhost:8000");
    let cliente = ClienteSocialHttp::new(url_servidor);

    // Invocamos el caso de uso que mide la latencia
    let (respuesta, latencia_ms) = verificar_conexion::calcular_latencia_ping(|| async move {
        cliente.hacer_ping().await
    })
    .await?;

    // Retornamos la estructura serializable en JSON
    Ok(PingResultadoUI {
        estado: respuesta.estado,
        mensaje: respuesta.mensaje,
        version: respuesta.version,
        timestamp: respuesta.timestamp,
        latencia_ms,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() { 
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![ping_servidor])
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicación tauri");
}