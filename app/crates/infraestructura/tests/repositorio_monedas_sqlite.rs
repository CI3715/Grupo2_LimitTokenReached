use aplicacion::puertos::RepositorioMonedas;
use infraestructura::{bd, RepositorioMonedasSqlite};
use rusqlite::Connection;

#[test]
fn distingue_monedas_de_fabrica_de_una_moneda_propia() {
    let mut conexion = Connection::open_in_memory().unwrap();
    bd::migrar(&mut conexion).unwrap();
    conexion
        .execute(
            "INSERT INTO moneda (codigo, nombre, simbolo, decimales) VALUES (?1, ?2, ?3, ?4)",
            ("PUNT", "Puntos de prueba", "P", 0),
        )
        .unwrap();

    let repositorio = RepositorioMonedasSqlite::new(conexion);
    let monedas = repositorio.obtener_monedas().unwrap();

    assert_eq!(monedas.len(), 5);
    for codigo in ["VES", "USD", "EUR", "USDT"] {
        let moneda = monedas.iter().find(|m| m.codigo == codigo).unwrap();
        assert!(moneda.es_del_catalogo, "{codigo} es una moneda de fábrica");
    }
    let propia = monedas.iter().find(|m| m.codigo == "PUNT").unwrap();
    assert!(!propia.es_del_catalogo);
    assert_eq!(propia.nombre, "Puntos de prueba");
    assert_eq!(propia.simbolo, "P");
    assert_eq!(propia.decimales, 0);
}
