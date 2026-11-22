use aplicacion::puertos::{RepositorioColaGastos, RepositorioSincronizacion};
use contrato::{
    CuentaGrupoSeleccionDTO, EventoCambioDTO, OperacionGastoOfflineDTO, RegistrarGastoRequestDTO,
};
use infraestructura::{RepositorioColaGastosSqlite, RepositorioSincronizacionSqlite, bd};
use rusqlite::Connection;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::path::PathBuf;

fn conexion_migrada(ruta: &std::path::Path) -> Connection {
    let mut conn = Connection::open(ruta).unwrap();
    bd::migrar(&mut conn).unwrap();
    conn
}

#[test]
fn flujo_general_sync_grupos_aplica_sin_duplicar_cursor_y_cola() {
    let ruta: PathBuf = std::env::temp_dir().join(format!(
        "cuentas-claras-flujo-sync-{}.db",
        uuid::Uuid::new_v4()
    ));
    let conn = conexion_migrada(&ruta);
    let conn_sync = conexion_migrada(&ruta);
    let conn_cola = conexion_migrada(&ruta);

    conn.execute(
        "INSERT INTO cuenta
         (id, nombre, tipo, moneda_codigo, saldo, creada_en, activa)
         VALUES ('00000000-0000-4000-8000-000000000010', 'Cuenta USD', 'Banco', 'USD', '100.00',
                 '2026-11-16', 1)",
        [],
    )
    .unwrap();

    let repo_sync = RepositorioSincronizacionSqlite::new(conn_sync);
    let repo_cola = RepositorioColaGastosSqlite::new(conn_cola);

    repo_sync
        .guardar_cuenta_grupo(&CuentaGrupoSeleccionDTO {
            grupo_id: "grupo-viaje".to_string(),
            cuenta_id: "00000000-0000-4000-8000-000000000010".to_string(),
            moneda: "USD".to_string(),
        })
        .unwrap();

    let evento = EventoCambioDTO {
        id_evento: "evento-hotel".to_string(),
        tipo: "GastoDeGrupoRegistrado".to_string(),
        grupo_id: "grupo-viaje".to_string(),
        gasto_id: "gasto-hotel".to_string(),
        concepto: "Hotel".to_string(),
        fecha: "2026-11-16T00:00:00Z".to_string(),
        moneda: "USD".to_string(),
        monto_parte: "33.34".to_string(),
        incluye_redondeo: true,
        posicion_cursor: 7,
    };

    let movimiento = aplicacion::casos_uso::aplicar_eventos_grupo::preparar_evento(
        &evento,
        Some("Viaje"),
        &repo_sync,
        &RepositorioCuentasPrueba,
    )
    .unwrap();

    let plan = match movimiento {
        aplicacion::casos_uso::aplicar_eventos_grupo::ResultadoPreparacion::Aplicar {
            movimiento,
            evento,
            efecto_saldo,
        } => aplicacion::puertos::LoteSincronizacionPlan {
            nuevo_cursor: 7,
            movimientos: vec![aplicacion::puertos::AplicacionGastoPlan {
                movimiento,
                evento: aplicacion::puertos::EventoGrupoPreparado {
                    id_evento: evento.id_evento,
                    grupo_id: evento.grupo_id,
                    gasto_id: evento.gasto_id,
                    concepto: evento.concepto,
                    fecha: evento.fecha,
                    moneda: evento.moneda,
                    monto_parte: evento.monto_parte,
                    incluye_redondeo: evento.incluye_redondeo,
                    posicion_cursor: evento.posicion_cursor,
                },
                efecto_saldo,
            }],
            pendientes: vec![],
        },
        _ => panic!("Se esperaba un movimiento aplicable"),
    };

    let aplicados = repo_sync.aplicar_lote_transaccional(&plan).unwrap();
    assert_eq!(aplicados, vec!["evento-hotel"]);
    assert_eq!(repo_sync.obtener_cursor().unwrap(), 7);
    assert!(repo_sync.evento_ya_aplicado("evento-hotel").unwrap());

    let segundo = repo_sync.aplicar_lote_transaccional(&plan).unwrap();
    assert!(segundo.is_empty());

    let saldo: String = conn
        .query_row(
            "SELECT saldo FROM cuenta WHERE id = '00000000-0000-4000-8000-000000000010'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(saldo.parse::<Decimal>().unwrap(), Decimal::new(6666, 2));

    repo_cola
        .guardar_pendiente(&OperacionGastoOfflineDTO {
            idempotency_key: "cola-1".to_string(),
            grupo_id: "grupo-viaje".to_string(),
            moneda: "USD".to_string(),
            datos: RegistrarGastoRequestDTO {
                concepto: "Cena".to_string(),
                monto: "40.00".to_string(),
                fecha: "2026-11-16".to_string(),
                pagador: "ana".to_string(),
                regla: "iguales".to_string(),
                participantes: vec!["ana".to_string()],
                valores: HashMap::new(),
            },
        })
        .unwrap();

    let estados = repo_cola.listar_estados_en_orden().unwrap();
    assert_eq!(estados.len(), 1);
    assert_eq!(estados[0].moneda, "USD");
    assert_eq!(estados[0].estado, "pendiente");

    repo_cola
        .marcar_rechazada("cola-1", "El grupo ya no existe.")
        .unwrap();
    let estados = repo_cola.listar_estados_en_orden().unwrap();
    assert_eq!(estados[0].estado, "rechazado");

    drop(conn);
    let _ = std::fs::remove_file(ruta);
}

struct RepositorioCuentasPrueba;

impl aplicacion::puertos::RepositorioCuentas for RepositorioCuentasPrueba {
    fn guardar_cuenta(
        &self,
        _request: &contrato::crear_cuenta::CrearCuentaRequest,
    ) -> Result<contrato::cuenta::CuentaDTO, String> {
        Err("No implementado".to_string())
    }

    fn obtener_cuentas(&self) -> Result<Vec<contrato::cuenta::CuentaDTO>, String> {
        Ok(vec![contrato::cuenta::CuentaDTO {
            id: "00000000-0000-4000-8000-000000000010".to_string(),
            nombre: "Cuenta USD".to_string(),
            tipo: contrato::cuenta::TipoCuentaDTO::Banco,
            moneda_codigo: "USD".to_string(),
            saldo: "100.00".to_string(),
            creada_en: "2026-11-16T00:00:00Z".to_string(),
            activa: true,
        }])
    }

    fn obtener_cuenta_por_id(&self, id: &str) -> Result<contrato::cuenta::CuentaDTO, String> {
        self.obtener_cuentas()?
            .into_iter()
            .find(|cuenta| cuenta.id == id)
            .ok_or_else(|| "Cuenta no encontrada".to_string())
    }
}
