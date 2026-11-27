//! Datos de prueba compartidos: escenario 2.6 de la Entrega 8 (convergencia Laptop/Teléfono).
//!
//! Cada archivo de `tests/` que lo necesite declara `mod comun;`.
//!
//! El escenario parte de 40 eventos con Efectivo USD = 151,67. Sin conexión:
//! - la laptop registra -20,00 y -5,00 (42 eventos, 126,67);
//! - el teléfono registra +50,00 y -10,00 (42 eventos, 191,67).
//!
//! Al juntar todo quedan 44 eventos y 166,67, sin importar el orden.
//!
//! Los eventos usan los nombres y campos reales que escribe la app en la tabla `eventos`
//! (`CuentaCreada`, `MovimientoRegistrado`) y se proyectan en `cuenta` y `movimiento`
//! igual que los repositorios SQLite. Los ids, dispositivos y horas son fijos para que
//! las pruebas sean reproducibles.

#![allow(dead_code)]

use infraestructura::bd;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde_json::{Value, json};
use uuid::Uuid;

pub const ESCALA_MONTOS: i64 = 100_000_000; // 10^8, igual que la app

pub const CUENTA_EFECTIVO: &str = "00000000-0000-4000-8000-000000000026";
pub const DISPOSITIVO_LAPTOP: &str = "00000000-0000-4000-8000-0000000000a1";
pub const DISPOSITIVO_TELEFONO: &str = "00000000-0000-4000-8000-0000000000a2";

const CATEGORIA_INGRESO: &str = "00000000-0000-0000-0000-000000000008"; // Salario
const CATEGORIA_GASTO: &str = "00000000-0000-0000-0000-000000000001"; // Alimentación

pub const SALDO_BASE: &str = "151.67";
pub const SALDO_SOLO_LAPTOP: &str = "126.67";
pub const SALDO_SOLO_TELEFONO: &str = "191.67";
pub const SALDO_CONVERGIDO: &str = "166.67";

pub const EVENTOS_BASE: i64 = 40;
pub const EVENTOS_CONVERGIDOS: i64 = 44;

/// Un evento tal como vive en la tabla `eventos`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoPrueba {
    pub id: String,
    pub tipo: String,
    pub registrado_en: String,
    pub dispositivo: String,
    pub datos: Value,
}

/// Una fila leída de la tabla `eventos`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaEvento {
    pub id: String,
    pub tipo: String,
    pub registrado_en: String,
    pub dispositivo: String,
    pub datos: String,
}

/// Estado completo de un libro: sirve para comprobar que "la base no cambió".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FotoLibro {
    pub eventos: Vec<FilaEvento>,
    pub saldos: Vec<(String, i64)>,
}

pub struct Escenario26 {
    pub base: Vec<EventoPrueba>,
    pub laptop: Vec<EventoPrueba>,
    pub telefono: Vec<EventoPrueba>,
}

fn id_evento(n: u128) -> String {
    Uuid::from_u128(0x2600_0000_0000_4000_8000_0000_0000_0000 + n).to_string()
}

fn id_movimiento(n: u128) -> String {
    Uuid::from_u128(0x2600_0000_0000_4000_8000_0001_0000_0000 + n).to_string()
}

/// Hora RFC 3339 con nanosegundos, como la escribe `chrono::to_rfc3339`.
fn marca(dia: u32, hora: u32, minuto: u32, segundo: u32) -> String {
    format!("2026-11-{dia:02}T{hora:02}:{minuto:02}:{segundo:02}.000000000+00:00")
}

fn cuenta_creada(n: u128, registrado_en: String, saldo_inicial: &str) -> EventoPrueba {
    EventoPrueba {
        id: id_evento(n),
        tipo: "CuentaCreada".to_string(),
        registrado_en,
        dispositivo: DISPOSITIVO_LAPTOP.to_string(),
        datos: json!({
            "id": CUENTA_EFECTIVO,
            "nombre": "Efectivo",
            "tipo": "efectivo",
            "moneda_codigo": "USD",
            "saldo_inicial": saldo_inicial,
            "creada_en": "2026-11-01",
        }),
    }
}

fn movimiento(
    n: u128,
    registrado_en: String,
    dispositivo: &str,
    tipo: &str,
    monto: &str,
) -> EventoPrueba {
    let categoria = if tipo == "Ingreso" {
        CATEGORIA_INGRESO
    } else {
        CATEGORIA_GASTO
    };
    EventoPrueba {
        id: id_evento(n),
        tipo: "MovimientoRegistrado".to_string(),
        registrado_en,
        dispositivo: dispositivo.to_string(),
        datos: json!({
            "id": id_movimiento(n),
            "cuenta_id": CUENTA_EFECTIVO,
            "tipo": tipo,
            "monto": monto,
            "fecha": "2026-11-10",
            "categoria_id": categoria,
            "nota": Value::Null,
        }),
    }
}

impl Escenario26 {
    pub fn nuevo() -> Self {
        // Base: la cuenta (100,00) + 38 ingresos de 1,00 + 1 ingreso de 13,67 = 40 eventos.
        let mut base = vec![cuenta_creada(0, marca(1, 8, 0, 0), "100.00")];
        for i in 1..=38_u32 {
            base.push(movimiento(
                u128::from(i),
                marca(1, 9, i, 0),
                DISPOSITIVO_LAPTOP,
                "Ingreso",
                "1.00",
            ));
        }
        base.push(movimiento(
            39,
            marca(1, 10, 0, 0),
            DISPOSITIVO_LAPTOP,
            "Ingreso",
            "13.67",
        ));

        let laptop = vec![
            movimiento(
                100,
                marca(10, 9, 0, 0),
                DISPOSITIVO_LAPTOP,
                "Gasto",
                "20.00",
            ),
            movimiento(101, marca(10, 9, 5, 0), DISPOSITIVO_LAPTOP, "Gasto", "5.00"),
        ];
        let telefono = vec![
            movimiento(
                200,
                marca(10, 9, 2, 0),
                DISPOSITIVO_TELEFONO,
                "Ingreso",
                "50.00",
            ),
            movimiento(
                201,
                marca(10, 9, 7, 0),
                DISPOSITIVO_TELEFONO,
                "Gasto",
                "10.00",
            ),
        ];
        Self {
            base,
            laptop,
            telefono,
        }
    }

    /// Los 44 eventos: base + laptop + teléfono.
    pub fn todos(&self) -> Vec<EventoPrueba> {
        let mut todos = self.base.clone();
        todos.extend(self.laptop.clone());
        todos.extend(self.telefono.clone());
        todos
    }

    /// Libro con los 40 eventos iniciales.
    pub fn libro_base(&self) -> Connection {
        libro_con(&self.base)
    }

    /// Libro de la laptop: base + sus 2 eventos (42).
    pub fn libro_laptop(&self) -> Connection {
        let mut eventos = self.base.clone();
        eventos.extend(self.laptop.clone());
        libro_con(&eventos)
    }

    /// Libro del teléfono: base + sus 2 eventos (42).
    pub fn libro_telefono(&self) -> Connection {
        let mut eventos = self.base.clone();
        eventos.extend(self.telefono.clone());
        libro_con(&eventos)
    }
}

/// Base SQLite en memoria, con todas las migraciones aplicadas y sin eventos.
pub fn base_vacia() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    bd::migrar(&mut conn).unwrap();
    conn
}

/// Base nueva con estos eventos ya aplicados.
pub fn libro_con(eventos: &[EventoPrueba]) -> Connection {
    let conn = base_vacia();
    aplicar(&conn, eventos);
    conn
}

/// Inserta eventos y actualiza las tablas `cuenta` y `movimiento`, como hace la app.
pub fn aplicar(conn: &Connection, eventos: &[EventoPrueba]) {
    for evento in eventos {
        conn.execute(
            "INSERT INTO eventos (id, tipo, registrado_en, dispositivo, datos)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                evento.id,
                evento.tipo,
                evento.registrado_en,
                evento.dispositivo,
                evento.datos.to_string()
            ],
        )
        .unwrap();
        match evento.tipo.as_str() {
            "CuentaCreada" => proyectar_cuenta(conn, &evento.datos),
            "MovimientoRegistrado" => proyectar_movimiento(conn, &evento.datos),
            otro => panic!("El constructor de prueba no sabe proyectar eventos '{otro}'"),
        }
    }
}

fn texto<'a>(datos: &'a Value, campo: &str) -> &'a str {
    datos[campo]
        .as_str()
        .unwrap_or_else(|| panic!("Falta el campo de texto '{campo}'"))
}

fn escalar(monto: &str) -> i64 {
    (monto.parse::<Decimal>().unwrap() * Decimal::from(ESCALA_MONTOS))
        .trunc()
        .to_i64()
        .unwrap()
}

fn proyectar_cuenta(conn: &Connection, datos: &Value) {
    conn.execute(
        "INSERT INTO cuenta (id, nombre, tipo, moneda_codigo, saldo, creada_en, activa)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
        params![
            texto(datos, "id"),
            texto(datos, "nombre"),
            texto(datos, "tipo"),
            texto(datos, "moneda_codigo"),
            escalar(texto(datos, "saldo_inicial")),
            texto(datos, "creada_en"),
        ],
    )
    .unwrap();
}

fn proyectar_movimiento(conn: &Connection, datos: &Value) {
    let monto = escalar(texto(datos, "monto"));
    let efecto = match texto(datos, "tipo") {
        "Ingreso" => monto,
        "Gasto" => -monto,
        otro => panic!("Tipo de movimiento inválido: {otro}"),
    };
    conn.execute(
        "INSERT INTO movimiento (id, cuenta_id, tipo, monto, fecha, categoria_id, nota)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            texto(datos, "id"),
            texto(datos, "cuenta_id"),
            texto(datos, "tipo"),
            monto,
            texto(datos, "fecha"),
            texto(datos, "categoria_id"),
            datos["nota"].as_str(),
        ],
    )
    .unwrap();
    conn.execute(
        "UPDATE cuenta SET saldo = saldo + ?1 WHERE id = ?2",
        params![efecto, texto(datos, "cuenta_id")],
    )
    .unwrap();
}

pub fn contar_eventos(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM eventos", [], |fila| fila.get(0))
        .unwrap()
}

/// Saldo que guarda la tabla `cuenta` (la proyección que muestra la app).
pub fn saldo_proyectado(conn: &Connection, cuenta: &str) -> Decimal {
    let crudo: i64 = conn
        .query_row(
            "SELECT saldo FROM cuenta WHERE id = ?1",
            params![cuenta],
            |fila| fila.get(0),
        )
        .unwrap();
    Decimal::new(crudo, 8).normalize()
}

fn leer_eventos(conn: &Connection, orden: &str) -> Vec<FilaEvento> {
    let sql =
        format!("SELECT id, tipo, registrado_en, dispositivo, datos FROM eventos ORDER BY {orden}");
    let mut sentencia = conn.prepare(&sql).unwrap();
    sentencia
        .query_map([], |fila| {
            Ok(FilaEvento {
                id: fila.get(0)?,
                tipo: fila.get(1)?,
                registrado_en: fila.get(2)?,
                dispositivo: fila.get(3)?,
                datos: fila.get(4)?,
            })
        })
        .unwrap()
        .map(|fila| fila.unwrap())
        .collect()
}

/// Saldo calculado solo desde los eventos, en el orden determinista del formato
/// (hora, dispositivo, id). Sirve de referencia para las pruebas de fusión.
pub fn saldo_recalculado(conn: &Connection, cuenta: &str) -> Decimal {
    let mut saldo = Decimal::ZERO;
    for fila in leer_eventos(conn, "registrado_en, dispositivo, id") {
        let datos: Value = serde_json::from_str(&fila.datos).unwrap();
        match fila.tipo.as_str() {
            "CuentaCreada" if texto(&datos, "id") == cuenta => {
                saldo += texto(&datos, "saldo_inicial").parse::<Decimal>().unwrap();
            }
            "MovimientoRegistrado" if texto(&datos, "cuenta_id") == cuenta => {
                let monto = texto(&datos, "monto").parse::<Decimal>().unwrap();
                match texto(&datos, "tipo") {
                    "Ingreso" => saldo += monto,
                    "Gasto" => saldo -= monto,
                    otro => panic!("Tipo de movimiento inválido: {otro}"),
                }
            }
            "MovimientoCorregido" if texto(&datos, "cuenta_id") == cuenta => {
                let variacion = texto(&datos, "variacion_monto").parse::<Decimal>().unwrap();
                match texto(&datos, "tipo_original") {
                    "Ingreso" => saldo += variacion,
                    "Gasto" => saldo -= variacion,
                    otro => panic!("Tipo original inválido: {otro}"),
                }
            }
            _ => {}
        }
    }
    saldo.normalize()
}

/// Foto del libro (eventos ordenados por id y saldos de cada cuenta). Dos libros con el
/// mismo conjunto de eventos dan la misma foto sin importar el orden de inserción.
pub fn foto(conn: &Connection) -> FotoLibro {
    let mut sentencia = conn
        .prepare("SELECT id, saldo FROM cuenta ORDER BY id")
        .unwrap();
    let saldos = sentencia
        .query_map([], |fila| {
            Ok((fila.get::<_, String>(0)?, fila.get::<_, i64>(1)?))
        })
        .unwrap()
        .map(|fila| fila.unwrap())
        .collect();
    FotoLibro {
        eventos: leer_eventos(conn, "id"),
        saldos,
    }
}

/// Texto de exportación sin cifrar según `docs/formato-exportacion.md`.
///
/// Es solo para pruebas: cuando exista la exportación real hay que reemplazarlo por ella.
pub fn a_texto_exportacion(conn: &Connection) -> String {
    let filas = leer_eventos(conn, "seq");
    let mut texto = format!(
        "VERSION=1.0\nFECHA_EXPORTACION=2026-11-25T10:00:00Z\nTOTAL_EVENTOS={}\n",
        filas.len()
    );
    for fila in filas {
        texto.push_str(&format!(
            "{}|{}|{}|{}|{}\n",
            fila.tipo, fila.id, fila.registrado_en, fila.dispositivo, fila.datos
        ));
    }
    texto
}
