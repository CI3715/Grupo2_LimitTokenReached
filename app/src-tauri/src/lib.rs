use aplicacion::casos_uso::verificar_conexion;
use infraestructura::ClienteSocialHttp;

#[tauri::command]
async fn ping_servidor() -> Result<String, String> {
    let url_servidor = "http://localhost:8000";
    let cliente = ClienteSocialHttp::new(url_servidor);

    // Se invoca la medición llamando al cliente de infraestructura
    verificar_conexion::calcular_latencia_ping(|| async move {
        let res = cliente.hacer_ping().await.map_err(|e| e.to_string())?;
        Ok((res.estado, res.mensaje, res.version, res.timestamp))
    })
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![ping_servidor])
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicación tauri");
}