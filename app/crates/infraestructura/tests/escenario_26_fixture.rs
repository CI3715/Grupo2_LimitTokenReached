//! Comprueba que el constructor del escenario 2.6 (`tests/comun`) arma los números del
//! enunciado: 40 eventos y 151,67 de base, 42 por dispositivo, y 44 y 166,67 al juntarlos.

mod comun;

use comun::*;
use rust_decimal::Decimal;
use std::collections::HashSet;

fn d(texto: &str) -> Decimal {
    texto.parse().unwrap()
}

#[test]
fn el_libro_base_tiene_40_eventos_y_151_67() {
    let escenario = Escenario26::nuevo();
    let conn = escenario.libro_base();

    assert_eq!(contar_eventos(&conn), EVENTOS_BASE);
    assert_eq!(saldo_proyectado(&conn, CUENTA_EFECTIVO), d(SALDO_BASE));
    assert_eq!(saldo_recalculado(&conn, CUENTA_EFECTIVO), d(SALDO_BASE));
}

#[test]
fn la_laptop_sola_queda_en_42_eventos_y_126_67() {
    let conn = Escenario26::nuevo().libro_laptop();

    assert_eq!(contar_eventos(&conn), 42);
    assert_eq!(
        saldo_proyectado(&conn, CUENTA_EFECTIVO),
        d(SALDO_SOLO_LAPTOP)
    );
    assert_eq!(
        saldo_recalculado(&conn, CUENTA_EFECTIVO),
        d(SALDO_SOLO_LAPTOP)
    );
}

#[test]
fn el_telefono_solo_queda_en_42_eventos_y_191_67() {
    let conn = Escenario26::nuevo().libro_telefono();

    assert_eq!(contar_eventos(&conn), 42);
    assert_eq!(
        saldo_proyectado(&conn, CUENTA_EFECTIVO),
        d(SALDO_SOLO_TELEFONO)
    );
    assert_eq!(
        saldo_recalculado(&conn, CUENTA_EFECTIVO),
        d(SALDO_SOLO_TELEFONO)
    );
}

#[test]
fn juntos_son_44_eventos_y_166_67() {
    let conn = libro_con(&Escenario26::nuevo().todos());

    assert_eq!(contar_eventos(&conn), EVENTOS_CONVERGIDOS);
    assert_eq!(
        saldo_proyectado(&conn, CUENTA_EFECTIVO),
        d(SALDO_CONVERGIDO)
    );
    assert_eq!(
        saldo_recalculado(&conn, CUENTA_EFECTIVO),
        d(SALDO_CONVERGIDO)
    );
}

#[test]
fn el_orden_en_que_se_juntan_no_cambia_el_resultado() {
    let escenario = Escenario26::nuevo();

    let mut laptop_primero = escenario.base.clone();
    laptop_primero.extend(escenario.laptop.clone());
    laptop_primero.extend(escenario.telefono.clone());

    let mut telefono_primero = escenario.base.clone();
    telefono_primero.extend(escenario.telefono.clone());
    telefono_primero.extend(escenario.laptop.clone());

    assert_eq!(
        foto(&libro_con(&laptop_primero)),
        foto(&libro_con(&telefono_primero))
    );
}

#[test]
fn la_foto_detecta_cambios_y_es_estable() {
    let escenario = Escenario26::nuevo();

    assert_eq!(foto(&escenario.libro_base()), foto(&escenario.libro_base()));
    assert_ne!(
        foto(&escenario.libro_base()),
        foto(&escenario.libro_laptop())
    );
}

#[test]
fn los_44_eventos_tienen_ids_unicos_y_horas_validas() {
    let todos = Escenario26::nuevo().todos();
    assert_eq!(todos.len() as i64, EVENTOS_CONVERGIDOS);

    let ids: HashSet<&str> = todos.iter().map(|evento| evento.id.as_str()).collect();
    assert_eq!(ids.len(), todos.len());

    for evento in &todos {
        uuid::Uuid::parse_str(&evento.id).unwrap();
        chrono::DateTime::parse_from_rfc3339(&evento.registrado_en).unwrap();
    }
}

#[test]
fn el_texto_de_exportacion_sigue_el_formato_documentado() {
    let conn = libro_con(&Escenario26::nuevo().todos());
    let texto = a_texto_exportacion(&conn);
    let lineas: Vec<&str> = texto.lines().collect();

    assert_eq!(lineas[0], "VERSION=1.0");
    assert!(lineas[1].starts_with("FECHA_EXPORTACION="));
    assert_eq!(lineas[2], "TOTAL_EVENTOS=44");
    assert_eq!(lineas.len(), 3 + 44);
    for linea in &lineas[3..] {
        assert_eq!(
            linea.splitn(5, '|').count(),
            5,
            "línea mal formada: {linea}"
        );
    }
}
