pub mod comandos;

use tauri::Manager;
use infraestructura::{bd, RepositorioCuentasSqlite, RepositorioMonedasSqlite, RepositorioMovimientosSqlite, RelojSistema};
use aplicacion::puertos::{RepositorioCuentas, RepositorioMonedas, RepositorioMovimientos, Reloj,};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;

            let ruta_db = dir.join("libro.db");

            // Abrir la base de datos SQLite y ejecutar migraciones
            let conn_cuentas = bd::abrir(&ruta_db)?;
            let conn_monedas = bd::abrir(&ruta_db)?;
            let conn_movimientos = bd::abrir(&ruta_db)?;

            // Registrar repositorios
            let repo_cuentas_sqlite = RepositorioCuentasSqlite::new(conn_cuentas);
            let state_cuentas: Box<dyn RepositorioCuentas> = Box::new(repo_cuentas_sqlite);
            app.manage(state_cuentas);

            let repo_monedas_sqlite = RepositorioMonedasSqlite::new(conn_monedas);
            let state_monedas: Box<dyn RepositorioMonedas> = Box::new(repo_monedas_sqlite);
            app.manage(state_monedas);

            

            let repo_movimientos_sqlite =
                RepositorioMovimientosSqlite::new(conn_movimientos);

            let state_movimientos: Box<dyn RepositorioMovimientos> =
                Box::new(repo_movimientos_sqlite);

            app.manage(state_movimientos);

            let reloj: Box<dyn Reloj> = Box::new(RelojSistema);
            app.manage(reloj);

            Ok(())
        })
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            comandos::ping::ping_servidor,
            comandos::cuentas::crear_cuenta,
            comandos::cuentas::listar_cuentas,
            comandos::moneda::listar_monedas,
            comandos::movimientos::registrar_movimiento,
            comandos::movimientos::corregir_movimiento,
            comandos::movimientos::listar_historial_movimientos,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}