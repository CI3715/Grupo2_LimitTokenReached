use infraestructura::bd;
use rusqlite::Connection;

#[test]
fn migracion_de_monedas_limita_decimales_a_ocho() {
    let mut conexion = Connection::open_in_memory().unwrap();

    bd::migrar(&mut conexion).unwrap();

    let resultado = conexion.execute(
        "INSERT INTO moneda
         (codigo, nombre, simbolo, decimales)
         VALUES ('TEST', 'Moneda de prueba', 'T', 9)",
        [],
    );

    assert!(
        resultado.is_err(),
        "SQLite debe rechazar más de 8 decimales"
    );
}

#[test]
fn migracion_de_monedas_acepta_ocho_decimales() {
    let mut conexion = Connection::open_in_memory().unwrap();

    bd::migrar(&mut conexion).unwrap();

    let resultado = conexion.execute(
        "INSERT INTO moneda
         (codigo, nombre, simbolo, decimales)
         VALUES ('TEST', 'Moneda de prueba', 'T', 8)",
        [],
    );

    assert!(resultado.is_ok());
}
