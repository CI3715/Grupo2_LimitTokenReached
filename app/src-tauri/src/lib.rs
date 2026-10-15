
pub mod comandos;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = comandos::bd::abrir(&dir.join("libro.db"))?;
            app.manage(comandos::bd::BaseDatos(Mutex::new(conn)));
            Ok(())
        })
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            comandos::ping::ping_servidor,
            comandos::bd::registrar_evento,
            comandos::cuentas::crear_cuenta,
            comandos::cuentas::listar_cuentas,
            comandos::moneda::listar_monedas
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}