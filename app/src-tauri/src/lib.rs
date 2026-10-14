pub mod comandos;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() { 
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![comandos::ping::ping_servidor])
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicación tauri");
}