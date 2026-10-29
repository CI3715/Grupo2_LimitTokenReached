use aplicacion::puertos::{RepositorioPatrimonio, RepositorioTasas};
use chrono::NaiveDate;
use contrato::tasa::RegistrarTasaRequest;
use infraestructura::{RepositorioPatrimonioSqlite, RepositorioTasasSqlite, bd};
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
            valor: "887.96000000123456789".to_string(),
            fecha: "2026-10-27".to_string(),
            fuente: "BCV".to_string(),
        })
        .unwrap();

    assert_eq!(tasa.valor, "887.96000000123456789");
    let encontradas = repo
        .obtener_tasas_hasta(NaiveDate::from_ymd_opt(2026, 10, 27).unwrap())
        .unwrap();
    assert!(
        encontradas
            .iter()
            .any(|tasa| tasa.valor == "887.96000000123456789")
    );
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
