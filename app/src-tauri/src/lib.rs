pub mod comandos;

use aplicacion::puertos::{
    Reloj, RepositorioCategorias, RepositorioCuentas, RepositorioMonedas, RepositorioMovimientos,
    RepositorioPatrimonio, RepositorioPreferencias, RepositorioTasas,
};
use infraestructura::{
    RelojSistema, RepositorioCategoriasSqlite, RepositorioCuentasSqlite, RepositorioMonedasSqlite,
    RepositorioMovimientosSqlite, RepositorioPatrimonioSqlite, RepositorioPreferenciasSqlite,
    RepositorioTasasSqlite, bd,
};
use tauri::Manager;

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
            let conn_categorias = bd::abrir(&ruta_db)?;
            let conn_tasas = bd::abrir(&ruta_db)?;
            let conn_patrimonio = bd::abrir(&ruta_db)?;
            let conn_preferencias = bd::abrir(&ruta_db)?;

            // Registrar repositorios
            let repo_cuentas_sqlite = RepositorioCuentasSqlite::new(conn_cuentas);
            let state_cuentas: Box<dyn RepositorioCuentas> = Box::new(repo_cuentas_sqlite);
            app.manage(state_cuentas);

            let repo_monedas_sqlite = RepositorioMonedasSqlite::new(conn_monedas);
            let state_monedas: Box<dyn RepositorioMonedas> = Box::new(repo_monedas_sqlite);
            app.manage(state_monedas);

            let repo_movimientos_sqlite = RepositorioMovimientosSqlite::new(conn_movimientos);

            let state_movimientos: Box<dyn RepositorioMovimientos> =
                Box::new(repo_movimientos_sqlite);

            app.manage(state_movimientos);

            let repo_categorias_sqlite = RepositorioCategoriasSqlite::new(conn_categorias);
            let state_categorias: Box<dyn RepositorioCategorias> = Box::new(repo_categorias_sqlite);

            app.manage(state_categorias);

            let state_tasas: Box<dyn RepositorioTasas> =
                Box::new(RepositorioTasasSqlite::new(conn_tasas));
            app.manage(state_tasas);

            let state_patrimonio: Box<dyn RepositorioPatrimonio> =
                Box::new(RepositorioPatrimonioSqlite::new(conn_patrimonio));
            app.manage(state_patrimonio);

            let state_preferencias: Box<dyn RepositorioPreferencias> =
                Box::new(RepositorioPreferenciasSqlite::new(conn_preferencias));
            app.manage(state_preferencias);

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
            comandos::moneda::crear_moneda,
            comandos::movimientos::registrar_movimiento,
            comandos::movimientos::corregir_movimiento,
            comandos::movimientos::listar_historial_movimientos,
            comandos::tasas::registrar_tasa,
            comandos::tasas::eliminar_tasa,
            comandos::tasas::listar_historial_tasas,
            comandos::patrimonio::consultar_patrimonio,
            comandos::preferencias::cambiar_moneda_base,
            comandos::categorias::crear_categoria,
            comandos::categorias::listar_categorias,
            comandos::moneda::eliminar_moneda,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}
