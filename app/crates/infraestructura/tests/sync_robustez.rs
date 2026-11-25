//! Robustez de la sincronización de gastos de grupo (E7: INV-13, INV-14, INV-15).
//!
//! Corre contra SQLite real en un archivo temporal, igual que `flujo_sync_grupos`.

use aplicacion::casos_uso::aplicar_eventos_grupo::{ResultadoPreparacion, preparar_evento};
use aplicacion::puertos::{
    AplicacionGastoPlan, LoteSincronizacionPlan, RepositorioCuentas, RepositorioSincronizacion,
};
use contrato::{CuentaGrupoSeleccionDTO, EventoCambioDTO};
use infraestructura::{RepositorioSincronizacionSqlite, bd};
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::path::{Path, PathBuf};

const CUENTA_A: &str = "00000000-0000-4000-8000-000000000010";
const CUENTA_B: &str = "00000000-0000-4000-8000-000000000011";
const ESCALA_MONTOS: i64 = 100_000_000; // 10^8

fn d(texto: &str) -> Decimal {
    texto.parse().unwrap()
}

fn conexion_migrada(ruta: &Path) -> Connection {
    let mut conn = Connection::open(ruta).unwrap();
    bd::migrar(&mut conn).unwrap();
    conn
}

/// Borra el archivo al final, cuando las conexiones ya se cerraron.
struct ArchivoTemporal(PathBuf);

impl Drop for ArchivoTemporal {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Entorno {
    conn: Connection,
    repo: RepositorioSincronizacionSqlite,
    // Va al final para que se borre después de cerrar las conexiones.
    _archivo: ArchivoTemporal,
}

impl Entorno {
    fn nuevo() -> Self {
        let ruta = std::env::temp_dir().join(format!(
            "cuentas-claras-robustez-sync-{}.db",
            uuid::Uuid::new_v4()
        ));
        let conn = conexion_migrada(&ruta);
        let repo = RepositorioSincronizacionSqlite::new(conexion_migrada(&ruta));
        Self {
            conn,
            repo,
            _archivo: ArchivoTemporal(ruta),
        }
    }

    fn crear_cuenta(&self, id: &str, saldo: &str) {
        let saldo_escalado = (d(saldo) * Decimal::from(ESCALA_MONTOS))
            .trunc()
            .to_i64()
            .unwrap();
        self.conn
            .execute(
                "INSERT INTO cuenta
                 (id, nombre, tipo, moneda_codigo, saldo, creada_en, activa)
                 VALUES (?1, 'Cuenta USD', 'Banco', 'USD', ?2, '2026-11-16', 1)",
                params![id, saldo_escalado],
            )
            .unwrap();
    }

    fn elegir_cuenta(&self, grupo: &str, cuenta: &str) {
        self.repo
            .guardar_cuenta_grupo(&CuentaGrupoSeleccionDTO {
                grupo_id: grupo.to_string(),
                cuenta_id: cuenta.to_string(),
                moneda: "USD".to_string(),
            })
            .unwrap();
    }

    fn saldo(&self, cuenta: &str) -> Decimal {
        let saldo_raw: i64 = self
            .conn
            .query_row(
                "SELECT saldo FROM cuenta WHERE id = ?1",
                params![cuenta],
                |fila| fila.get(0),
            )
            .unwrap();
        Decimal::new(saldo_raw, 8).normalize()
    }

    fn contar(&self, tabla: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {tabla}");
        self.conn.query_row(&sql, [], |fila| fila.get(0)).unwrap()
    }

    fn contar_eventos(&self, tipo: &str) -> i64 {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM eventos WHERE tipo = ?1",
                params![tipo],
                |fila| fila.get(0),
            )
            .unwrap()
    }
}

struct CuentasPrueba;

fn cuenta_dto(id: &str) -> contrato::cuenta::CuentaDTO {
    contrato::cuenta::CuentaDTO {
        id: id.to_string(),
        nombre: "Cuenta USD".to_string(),
        tipo: contrato::cuenta::TipoCuentaDTO::Banco,
        moneda_codigo: "USD".to_string(),
        saldo: "100.00".to_string(),
        creada_en: "2026-11-16T00:00:00Z".to_string(),
        activa: true,
    }
}

impl RepositorioCuentas for CuentasPrueba {
    fn guardar_cuenta(
        &self,
        _request: &contrato::crear_cuenta::CrearCuentaRequest,
    ) -> Result<contrato::cuenta::CuentaDTO, String> {
        Err("No implementado".to_string())
    }

    fn obtener_cuentas(&self) -> Result<Vec<contrato::cuenta::CuentaDTO>, String> {
        Ok(vec![cuenta_dto(CUENTA_A), cuenta_dto(CUENTA_B)])
    }

    fn obtener_cuenta_por_id(&self, id: &str) -> Result<contrato::cuenta::CuentaDTO, String> {
        self.obtener_cuentas()?
            .into_iter()
            .find(|cuenta| cuenta.id == id)
            .ok_or_else(|| "Cuenta no encontrada".to_string())
    }
}

fn evento(id: &str, grupo: &str, posicion: u64, monto: &str) -> EventoCambioDTO {
    EventoCambioDTO {
        id_evento: id.to_string(),
        tipo: "GastoDeGrupoRegistrado".to_string(),
        grupo_id: grupo.to_string(),
        gasto_id: format!("gasto-{id}"),
        concepto: "Hotel".to_string(),
        fecha: "2026-11-16T00:00:00Z".to_string(),
        moneda: "USD".to_string(),
        monto_parte: monto.to_string(),
        incluye_redondeo: false,
        posicion_cursor: posicion,
    }
}

/// Arma el plan igual que `sincronizar_cambios`: lo ya aplicado no entra.
fn plan_de(
    eventos: &[EventoCambioDTO],
    nuevo_cursor: u64,
    repo: &dyn RepositorioSincronizacion,
) -> LoteSincronizacionPlan {
    let mut movimientos = Vec::new();
    for evento in eventos {
        match preparar_evento(evento, Some("Viaje"), repo, &CuentasPrueba).unwrap() {
            ResultadoPreparacion::Aplicar {
                movimiento,
                evento: preparado,
                efecto_saldo,
            } => movimientos.push(AplicacionGastoPlan {
                movimiento,
                evento: preparado,
                efecto_saldo,
            }),
            ResultadoPreparacion::YaAplicado => {}
            ResultadoPreparacion::Pendiente(_) => {
                panic!("Se esperaba una parte aplicable, no pendiente")
            }
        }
    }
    LoteSincronizacionPlan {
        nuevo_cursor,
        movimientos,
        pendientes: vec![],
    }
}

#[test]
fn cursor_en_cero_no_duplica_movimientos_ni_altera_el_saldo() {
    let e = Entorno::nuevo();
    e.crear_cuenta(CUENTA_A, "100.00");
    e.elegir_cuenta("grupo-viaje", CUENTA_A);
    let eventos = vec![
        evento("evento-hotel", "grupo-viaje", 1, "33.33"),
        evento("evento-cena", "grupo-viaje", 2, "15.00"),
    ];

    let plan = plan_de(&eventos, 2, &e.repo);
    assert_eq!(e.repo.aplicar_lote_transaccional(&plan).unwrap().len(), 2);
    assert_eq!(e.saldo(CUENTA_A), d("51.67"));

    // El usuario reinicia el cursor y el servidor reenvía los mismos eventos.
    e.repo.guardar_cursor(0).unwrap();
    for _ in 0..3 {
        let plan = plan_de(&eventos, 2, &e.repo);
        assert!(plan.movimientos.is_empty(), "Lo ya aplicado no debe repetirse");
        assert!(e.repo.aplicar_lote_transaccional(&plan).unwrap().is_empty());
    }

    assert_eq!(e.repo.obtener_cursor().unwrap(), 2);
    assert_eq!(e.saldo(CUENTA_A), d("51.67"));
    assert_eq!(e.contar("movimiento"), 2);
    assert_eq!(e.contar("eventos_grupo_aplicados"), 2);
    assert_eq!(e.contar_eventos("GastoDeGrupoAplicado"), 2);
}

#[test]
fn mismo_evento_repetido_dentro_de_un_lote_se_aplica_una_vez() {
    let e = Entorno::nuevo();
    e.crear_cuenta(CUENTA_A, "100.00");
    e.elegir_cuenta("grupo-viaje", CUENTA_A);

    let plan = plan_de(
        &[
            evento("evento-hotel", "grupo-viaje", 1, "33.34"),
            evento("evento-hotel", "grupo-viaje", 1, "33.34"),
        ],
        1,
        &e.repo,
    );
    assert_eq!(plan.movimientos.len(), 2);

    let aplicados = e.repo.aplicar_lote_transaccional(&plan).unwrap();
    assert_eq!(aplicados, vec!["evento-hotel"]);
    assert_eq!(e.saldo(CUENTA_A), d("66.66"));
    assert_eq!(e.contar("movimiento"), 1);
}

#[test]
fn lote_interrumpido_no_deja_nada_a_medias_y_el_siguiente_lo_completa() {
    let e = Entorno::nuevo();
    e.crear_cuenta(CUENTA_A, "100.00");
    e.elegir_cuenta("grupo-viaje", CUENTA_A);

    // El segundo movimiento apunta a una cuenta que no existe en la base:
    // falla a mitad del lote, después de haber procesado el primero.
    let mut plan = plan_de(
        &[evento("evento-1", "grupo-viaje", 1, "10.00")],
        2,
        &e.repo,
    );
    let mut segundo = plan.movimientos[0].clone();
    segundo.movimiento.id = uuid::Uuid::new_v4();
    segundo.movimiento.cuenta_id = uuid::Uuid::parse_str(CUENTA_B).unwrap();
    segundo.movimiento.monto = d("20.00");
    segundo.efecto_saldo = d("-20.00");
    segundo.evento.id_evento = "evento-2".to_string();
    segundo.evento.gasto_id = "gasto-2".to_string();
    segundo.evento.monto_parte = "20.00".to_string();
    segundo.evento.posicion_cursor = 2;
    plan.movimientos.push(segundo);

    assert!(e.repo.aplicar_lote_transaccional(&plan).is_err());

    // Nada quedó a medias: ni cursor, ni saldo, ni movimientos, ni eventos.
    assert_eq!(e.repo.obtener_cursor().unwrap(), 0);
    assert_eq!(e.saldo(CUENTA_A), d("100.00"));
    assert_eq!(e.contar("movimiento"), 0);
    assert_eq!(e.contar("eventos_grupo_aplicados"), 0);
    assert_eq!(e.contar_eventos("MovimientoRegistrado"), 0);
    assert!(!e.repo.evento_ya_aplicado("evento-1").unwrap());

    // La siguiente sincronización, ya sin el problema, termina el trabajo.
    e.crear_cuenta(CUENTA_B, "100.00");
    let aplicados = e.repo.aplicar_lote_transaccional(&plan).unwrap();
    assert_eq!(aplicados, vec!["evento-1", "evento-2"]);
    assert_eq!(e.repo.obtener_cursor().unwrap(), 2);
    assert_eq!(e.saldo(CUENTA_A), d("90.00"));
    assert_eq!(e.saldo(CUENTA_B), d("80.00"));
    assert_eq!(e.contar("movimiento"), 2);

    // Y repetirla no duplica nada.
    assert!(e.repo.aplicar_lote_transaccional(&plan).unwrap().is_empty());
    assert_eq!(e.contar("movimiento"), 2);
}