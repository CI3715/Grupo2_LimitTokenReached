pub mod comandos;

mod secretos;

use aplicacion::puertos::{
    AlmacenSecretos, CifradorRespaldo, ClienteSocial, Reloj, RepositorioCategorias,
    RepositorioColaGastos, RepositorioCuentas, RepositorioEventos, RepositorioMonedas,
    RepositorioMovimientos, RepositorioPatrimonio, RepositorioPreferencias,
    RepositorioSincronizacion, RepositorioTasas, SerializadorLibro,
};
use infraestructura::{
    CifradorRespaldoAesGcm, ClienteSocialHttp, RelojSistema, RepositorioCategoriasSqlite,
    RepositorioColaGastosSqlite, RepositorioCuentasSqlite, RepositorioEventosSqlite,
    RepositorioMonedasSqlite, RepositorioMovimientosSqlite, RepositorioPatrimonioSqlite,
    RepositorioPreferenciasSqlite, RepositorioSincronizacionSqlite, RepositorioTasasSqlite,
    SerializadorLibroPlano, bd,
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
            let conn_sincronizacion = bd::abrir(&ruta_db)?;
            let conn_cola = bd::abrir(&ruta_db)?;

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

            let state_sincronizacion: Box<dyn RepositorioSincronizacion> =
                Box::new(RepositorioSincronizacionSqlite::new(conn_sincronizacion));
            app.manage(state_sincronizacion);

            let state_cola: Box<dyn RepositorioColaGastos> =
                Box::new(RepositorioColaGastosSqlite::new(conn_cola));
            app.manage(state_cola);

            let reloj: Box<dyn Reloj> = Box::new(RelojSistema);
            app.manage(reloj);

            // Entrega 8: libro de eventos, formato plano y cifrado de respaldos.
            let conn_eventos = bd::abrir(&ruta_db)?;
            let state_eventos: Box<dyn RepositorioEventos> =
                Box::new(RepositorioEventosSqlite::new(conn_eventos));
            app.manage(state_eventos);

            let state_serializador: Box<dyn SerializadorLibro> =
                Box::new(SerializadorLibroPlano::new());
            app.manage(state_serializador);

            let state_cifrador: Box<dyn CifradorRespaldo> = Box::new(CifradorRespaldoAesGcm::new());
            app.manage(state_cifrador);

            // Entrega 8: credenciales de Git y frase de paso del destino configurado.
            let state_secretos: Box<dyn AlmacenSecretos> =
                Box::new(secretos::AlmacenSecretosSistema);
            app.manage(state_secretos);

            let url_servidor = option_env!("SERVER_HOST_IP").unwrap_or("http://localhost:8000");
            let cliente_social: Box<dyn ClienteSocial> =
                Box::new(ClienteSocialHttp::new(url_servidor));
            app.manage(cliente_social);

            // Iniciar sincronización automática al arrancar y cada 5 minutos
            comandos::sync::iniciar_sincronizacion_automatica(app.handle().clone());

            Ok(())
        })
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            comandos::ping::ping_servidor,
            comandos::autenticacion::registrar_usuario,
            comandos::autenticacion::iniciar_sesion,
            comandos::autenticacion::obtener_perfil,
            comandos::autenticacion::cerrar_sesion,
            comandos::social::buscar_usuario,
            comandos::social::invitar_amistad,
            comandos::social::responder_invitacion_amistad,
            comandos::social::listar_amistades,
            comandos::social::eliminar_amistad,
            comandos::social::listar_grupos,
            comandos::social::crear_grupo,
            comandos::social::invitar_miembro_grupo,
            comandos::social::responder_invitacion_grupo,
            comandos::social::registrar_gasto,
            comandos::social::obtener_detalle_grupo,
            comandos::sync::sincronizar_grupos,
            comandos::sync::elegir_cuenta_grupo,
            comandos::sync::obtener_cuenta_grupo,
            comandos::sync::listar_partes_grupo_pendientes,
            comandos::sync::listar_cola_gastos_grupo,
            comandos::sync::estado_sincronizacion,
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
            comandos::previsualizacion::previsualizar_division,
            comandos::respaldo::exportar_libro,
            comandos::respaldo::importar_libro,
            comandos::respaldo::sincronizar_carpeta_libro,
            comandos::respaldo::sincronizar_git_libro,
            comandos::respaldo::configurar_destino_sync,
            comandos::respaldo::obtener_destino_sync,
            comandos::respaldo::desconfigurar_destino_sync,
            comandos::respaldo::sincronizar_destino_configurado,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}
