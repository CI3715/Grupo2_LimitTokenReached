use chrono::NaiveDate;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use uuid::Uuid;

use aplicacion::puertos::RepositorioMovimientos;
use dominio::movimiento::{Ajuste, Movimiento, TipoMovimiento};
use infraestructura::bd;
use infraestructura::RepositorioMovimientosSqlite;

fn crear_repositorio() -> RepositorioMovimientosSqlite {
    let mut conexion = Connection::open_in_memory()
        .expect("No se pudo crear la base SQLite en memoria");

    bd::migrar(&mut conexion)
        .expect("No se pudieron ejecutar las migraciones");

    let cuenta_id = Uuid::from_u128(1000);

    conexion
        .execute(
            "
            INSERT INTO cuenta (id, nombre, tipo, moneda_codigo, saldo)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                cuenta_id.to_string(),
                "Cuenta de prueba",
                "Efectivo",
                "USD",
                "100.00",
            ],
        )
        .expect("No se pudo crear la cuenta de prueba");

    RepositorioMovimientosSqlite::new(conexion)
}

fn consultar_saldo(repo: &RepositorioMovimientosSqlite, cuenta_id: &Uuid) -> Decimal {
    let conexion = repo
        .conexion
        .lock()
        .expect("No se pudo bloquear la conexión");

    let saldo: String = conexion
        .query_row(
            "SELECT saldo FROM cuenta WHERE id = ?1",
            params![cuenta_id.to_string()],
            |fila| fila.get(0),
        )
        .expect("No se pudo consultar el saldo");

    saldo
        .parse::<Decimal>()
        .expect("El saldo almacenado no es un decimal válido")
}

#[test]
fn guardar_ingreso_aumenta_el_saldo_y_persiste_el_movimiento() {
    let repo = crear_repositorio();

    let cuenta_id = Uuid::from_u128(1000);
    let movimiento_id = Uuid::from_u128(2000);
    let categoria_ingreso_id = Uuid::from_u128(8);

    let movimiento = Movimiento {
        id: movimiento_id,
        cuenta_id,
        tipo: TipoMovimiento::Ingreso,
        monto: Decimal::new(50, 0),
        fecha: NaiveDate::from_ymd_opt(2026, 10, 22)
            .expect("Fecha de prueba inválida"),
        categoria_id: categoria_ingreso_id,
        nota: Some("Ingreso de prueba".to_string()),
    };

    let dto = repo
        .guardar_movimiento(&movimiento)
        .expect("No se pudo guardar el ingreso");

    assert_eq!(dto.id, movimiento_id.to_string());
    assert_eq!(dto.cuenta_id, cuenta_id.to_string());
    assert_eq!(dto.monto, "50");
    assert!(!dto.es_correccion);

    assert_eq!(
        consultar_saldo(&repo, &cuenta_id),
        Decimal::new(150, 0)
    );

    let movimiento_guardado = repo
        .obtener_movimiento_por_id(&movimiento_id)
        .expect("No se pudo consultar el movimiento guardado");

    assert_eq!(movimiento_guardado, movimiento);
}

#[test]
fn guardar_gasto_disminuye_el_saldo() {
    let repo = crear_repositorio();

    let cuenta_id = Uuid::from_u128(1000);
    let movimiento_id = Uuid::from_u128(3000);
    let categoria_gasto_id = Uuid::from_u128(1);

    let movimiento = Movimiento {
        id: movimiento_id,
        cuenta_id,
        tipo: TipoMovimiento::Gasto,
        monto: Decimal::new(30, 0),
        fecha: NaiveDate::from_ymd_opt(2026, 10, 22)
            .expect("Fecha de prueba inválida"),
        categoria_id: categoria_gasto_id,
        nota: Some("Gasto de prueba".to_string()),
    };

    repo.guardar_movimiento(&movimiento)
        .expect("No se pudo guardar el gasto");

    assert_eq!(
        consultar_saldo(&repo, &cuenta_id),
        Decimal::new(70, 0)
    );
}

#[test]
fn guardar_ajuste_actualiza_el_saldo_y_persiste_la_correccion() {
    let repo = crear_repositorio();

    let cuenta_id = Uuid::from_u128(1000);
    let movimiento_id = Uuid::from_u128(4000);
    let ajuste_id = Uuid::from_u128(5000);
    let categoria_ingreso_id = Uuid::from_u128(8);

    let movimiento = Movimiento {
        id: movimiento_id,
        cuenta_id,
        tipo: TipoMovimiento::Ingreso,
        monto: Decimal::new(50, 0),
        fecha: NaiveDate::from_ymd_opt(2026, 10, 22)
            .expect("Fecha de prueba inválida"),
        categoria_id: categoria_ingreso_id,
        nota: Some("Ingreso corregible".to_string()),
    };

    repo.guardar_movimiento(&movimiento)
        .expect("No se pudo guardar el movimiento original");

    assert_eq!(
        consultar_saldo(&repo, &cuenta_id),
        Decimal::new(150, 0)
    );

    let ajuste = Ajuste {
        id: ajuste_id,
        movimiento_original_id: movimiento_id,
        cuenta_id,
        tipo_original: TipoMovimiento::Ingreso,
        variacion_monto: Decimal::new(-20, 0),
    };

    let dto = repo
        .guardar_ajuste(&ajuste)
        .expect("No se pudo guardar el ajuste");

    assert_eq!(dto.id, ajuste_id.to_string());
    assert!(dto.es_correccion);
    assert_eq!(
        dto.movimiento_original_id,
        Some(movimiento_id.to_string())
    );

    assert_eq!(
        consultar_saldo(&repo, &cuenta_id),
        Decimal::new(130, 0)
    );

    let ajustes = repo
        .obtener_ajustes_de_movimiento(&movimiento_id)
        .expect("No se pudieron consultar los ajustes");

    assert_eq!(ajustes.len(), 1);
    assert_eq!(ajustes[0], ajuste);
}

#[test]
fn obtener_categorias_devuelve_las_categorias_de_la_migracion() {
    let repo = crear_repositorio();

    let categorias = repo
        .obtener_categorias()
        .expect("No se pudieron consultar las categorías");

    assert_eq!(categorias.len(), 11);

    assert!(
        categorias
            .iter()
            .any(|categoria| categoria.id == Uuid::from_u128(1))
    );

    assert!(
        categorias
            .iter()
            .any(|categoria| categoria.id == Uuid::from_u128(8))
    );
}