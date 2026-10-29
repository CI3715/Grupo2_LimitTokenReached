use aplicacion::puertos::{RepositorioMonedas, RepositorioPatrimonio, RepositorioTasas};
use chrono::NaiveDate;
use contrato::moneda::CrearMonedaRequest;
use contrato::tasa::RegistrarTasaRequest;
use infraestructura::{
    RepositorioMonedasSqlite, RepositorioPatrimonioSqlite, RepositorioTasasSqlite, bd,
};
use rusqlite::Connection;

fn conexion_migrada() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    bd::migrar(&mut conn).unwrap();
    conn
}

#[test]
fn registrar_tasa_preserva_precision_y_registra_evento() {
    let repo = RepositorioTasasSqlite::new(conexion_migrada());
    let tasa = repo
        .guardar_tasa(&RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: "887.96000000".to_string(),
            fecha: "2026-10-27".to_string(),
            fuente: "BCV".to_string(),
        })
        .unwrap();

    assert_eq!(tasa.valor, "887.96000000");
    let encontradas = repo
        .obtener_tasas_hasta(NaiveDate::from_ymd_opt(2026, 10, 27).unwrap())
        .unwrap();
    assert!(encontradas.iter().any(|tasa| tasa.valor == "887.96000000"));
}

#[test]
fn consulta_saldo_historico_aplica_movimientos_y_ajustes_por_fecha() {
    let conn = conexion_migrada();
    conn.execute(
        "INSERT INTO cuenta (id, nombre, tipo, moneda_codigo, saldo, creada_en)
         VALUES ('cuenta-1', 'Ahorros', 'Banco', 'USD', '135.00', '2026-01-01')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO movimiento (id, cuenta_id, tipo, monto, fecha, categoria_id)
         VALUES
            ('mov-ingreso', 'cuenta-1', 'Ingreso', '50.00', '2026-01-05',
             '00000000-0000-0000-0000-000000000008'),
            ('mov-gasto', 'cuenta-1', 'Gasto', '20.00', '2026-01-07',
             '00000000-0000-0000-0000-000000000001')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO ajuste
             (id, movimiento_original_id, cuenta_id, tipo_original, variacion_monto, fecha)
         VALUES ('ajuste-1', 'mov-gasto', 'cuenta-1', 'Gasto', '-5.00', '2026-01-10')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO cuenta (id, nombre, tipo, moneda_codigo, saldo, creada_en)
         VALUES ('cuenta-futura', 'Futura', 'Banco', 'USD', '10.00', '2026-02-01')",
        [],
    )
    .unwrap();

    let repo = RepositorioPatrimonioSqlite::new(conn);
    let saldos = repo
        .obtener_saldos_a_fecha(NaiveDate::from_ymd_opt(2026, 1, 8).unwrap())
        .unwrap();
    assert_eq!(saldos.len(), 1);
    assert_eq!(saldos[0].saldo, "130");

    let saldos = repo
        .obtener_saldos_a_fecha(NaiveDate::from_ymd_opt(2026, 1, 10).unwrap())
        .unwrap();
    assert_eq!(saldos[0].saldo, "135");
}

#[test]
fn eliminar_tasa_marca_la_tasa_sin_borrarla() {
    let conn = conexion_migrada();
    let repo = RepositorioTasasSqlite::new(conn);

    let tasa = repo
        .guardar_tasa(&RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: "900.12345678".to_string(),
            fecha: "2026-10-28".to_string(),
            fuente: "BCV".to_string(),
        })
        .unwrap();

    repo.eliminar_tasa(&tasa.id).unwrap();

    let historial = repo
        .obtener_historial_tasas_hasta(NaiveDate::from_ymd_opt(2026, 10, 28).unwrap())
        .unwrap();

    let eliminada = historial
        .iter()
        .find(|registro| registro.id == tasa.id)
        .expect("La tasa debe permanecer en el historial");

    assert!(eliminada.eliminada);

    let activas = repo
        .obtener_tasas_hasta(NaiveDate::from_ymd_opt(2026, 10, 28).unwrap())
        .unwrap();

    assert!(
        activas.iter().all(|registro| registro.id != tasa.id),
        "La tasa eliminada no debe aparecer entre las tasas activas"
    );
}

#[test]
fn historial_respeta_la_fecha_limite() {
    let repo = RepositorioTasasSqlite::new(conexion_migrada());

    let tasa = repo
        .guardar_tasa(&RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: "900".to_string(),
            fecha: "2026-10-28".to_string(),
            fuente: "BCV".to_string(),
        })
        .unwrap();

    let historial = repo
        .obtener_historial_tasas_hasta(NaiveDate::from_ymd_opt(2026, 10, 27).unwrap())
        .unwrap();

    assert!(
        historial.iter().all(|registro| registro.id != tasa.id),
        "La tasa del 2026-10-28 no debe aparecer al consultar hasta el 2026-10-27"
    );
}

#[test]
fn usa_la_tasa_mas_reciente_cuando_hay_dos_en_la_misma_fecha() {
    let repo = RepositorioTasasSqlite::new(conexion_migrada());

    let primera_tasa = repo
        .guardar_tasa(&RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: "900".to_string(),
            fecha: "2026-10-28".to_string(),
            fuente: "Fuente anterior".to_string(),
        })
        .unwrap();

    let segunda_tasa = repo
        .guardar_tasa(&RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: "905".to_string(),
            fecha: "2026-10-28".to_string(),
            fuente: "Fuente reciente".to_string(),
        })
        .unwrap();

    let tasas = repo
        .obtener_tasas_hasta(NaiveDate::from_ymd_opt(2026, 10, 28).unwrap())
        .unwrap();

    let tasas_del_par: Vec<_> = tasas
        .iter()
        .filter(|tasa| tasa.id == primera_tasa.id || tasa.id == segunda_tasa.id)
        .collect();

    assert_eq!(
        tasas_del_par.len(),
        2,
        "Las dos tasas del mismo par y fecha deben coexistir"
    );

    assert_eq!(
        tasas_del_par[0].id, segunda_tasa.id,
        "La tasa registrada más recientemente debe aparecer primero"
    );

    assert_eq!(tasas_del_par[0].valor, "905");
    assert_eq!(tasas_del_par[0].fuente, "Fuente reciente");
}

#[test]
fn guarda_moneda_propia_y_la_lista() {
    let conexion = conexion_migrada();
    let repo = RepositorioMonedasSqlite::new(conexion);

    let request = CrearMonedaRequest {
        codigo: "GBP".to_string(),
        nombre: "Libra Esterlina".to_string(),
        simbolo: "£".to_string(),
        decimales: 2,
    };

    let creada = repo.guardar_moneda(&request).unwrap();

    assert_eq!(creada.codigo, "GBP");
    assert_eq!(creada.nombre, "Libra Esterlina");
    assert_eq!(creada.simbolo, "£");
    assert_eq!(creada.decimales, 2);
    assert!(!creada.es_del_catalogo);

    let monedas = repo.obtener_monedas().unwrap();

    let encontrada = monedas
        .iter()
        .find(|moneda| moneda.codigo == "GBP")
        .expect("La moneda propia debe aparecer en el catálogo");

    assert_eq!(encontrada.nombre, "Libra Esterlina");
    assert_eq!(encontrada.simbolo, "£");
    assert_eq!(encontrada.decimales, 2);
    assert!(!encontrada.es_del_catalogo);
}

#[test]
fn moneda_propia_puede_ser_referenciada_por_una_cuenta() {
    let conexion = conexion_migrada();
    let repo_monedas = RepositorioMonedasSqlite::new(conexion);

    let request = CrearMonedaRequest {
        codigo: "GBP".to_string(),
        nombre: "Libra Esterlina".to_string(),
        simbolo: "£".to_string(),
        decimales: 2,
    };

    repo_monedas.guardar_moneda(&request).unwrap();

    let monedas = repo_monedas.obtener_monedas().unwrap();

    assert!(monedas.iter().any(|moneda| moneda.codigo == "GBP"));
}

#[test]
fn no_elimina_moneda_de_fabrica_en_sqlite() {
    let repo = RepositorioMonedasSqlite::new(conexion_migrada());

    let resultado = repo.eliminar_moneda("USD");

    assert!(resultado.is_err());
    assert_eq!(
        resultado.unwrap_err(),
        "Las monedas de fábrica no se pueden eliminar."
    );
}

#[test]
fn elimina_moneda_propia_no_utilizada_en_sqlite() {
    let repo = RepositorioMonedasSqlite::new(conexion_migrada());

    repo.guardar_moneda(&CrearMonedaRequest {
        codigo: "GBP".to_string(),
        nombre: "Libra Esterlina".to_string(),
        simbolo: "£".to_string(),
        decimales: 2,
    })
    .unwrap();

    repo.eliminar_moneda("GBP").unwrap();

    let monedas = repo.obtener_monedas().unwrap();

    assert!(
        monedas.iter().all(|moneda| moneda.codigo != "GBP"),
        "La moneda propia eliminada no debe aparecer"
    );
}

#[test]
fn no_elimina_moneda_utilizada_por_una_cuenta() {
    let conn = conexion_migrada();

    let repo_monedas = RepositorioMonedasSqlite::new(conn);

    repo_monedas
        .guardar_moneda(&CrearMonedaRequest {
            codigo: "GBP".to_string(),
            nombre: "Libra Esterlina".to_string(),
            simbolo: "£".to_string(),
            decimales: 2,
        })
        .unwrap();

    let conexion = repo_monedas.conexion.lock().unwrap();

    conexion
        .execute(
            "INSERT INTO cuenta
             (id, nombre, tipo, moneda_codigo, saldo, creada_en)
             VALUES ('cuenta-gbp', 'Cuenta GBP', 'Banco', 'GBP', '0', '2026-01-01')",
            [],
        )
        .unwrap();

    drop(conexion);

    let resultado = repo_monedas.eliminar_moneda("GBP");

    assert!(resultado.is_err());
    assert!(
        resultado
            .unwrap_err()
            .contains("una cuenta, una tasa o la moneda base")
    );
}
