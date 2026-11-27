use crate::bd;
use chrono::Utc;
use rusqlite::Connection;
use rusqlite::params;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde_json::json;
use std::sync::Mutex;
use uuid::Uuid;

use aplicacion::puertos::RepositorioCuentas;
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::cuenta::{CuentaDTO, TipoCuentaDTO};

const ESCALA_MONTOS: i64 = 100_000_000; // 10^8

fn decimal_a_escalado(monto: Decimal) -> Result<i64, String> {
    let escalado = monto
        .checked_mul(Decimal::from(ESCALA_MONTOS))
        .ok_or_else(|| "Desbordamiento al escalar el monto".to_string())?;

    escalado
        .trunc()
        .to_i64()
        .ok_or_else(|| "El monto excede el rango permitido por SQLite (i64)".to_string())
}

fn escalado_a_decimal(valor: i64) -> Decimal {
    Decimal::new(valor, 8).normalize()
}

pub struct RepositorioCuentasSqlite {
    pub conexion: Mutex<Connection>,
}

impl RepositorioCuentasSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

impl RepositorioCuentas for RepositorioCuentasSqlite {
    fn guardar_cuenta(&self, req: &CrearCuentaRequest) -> Result<CuentaDTO, String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;

        // 1. Iniciar transacción explícita
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let tipo_str = serde_json::to_string(&req.tipo)
            .map_err(|e| e.to_string())?
            .trim_matches('"')
            .to_string();

        let saldo_decimal = req
            .saldo_inicial
            .trim()
            .parse::<Decimal>()
            .map_err(|e| format!("Saldo inicial inválido: {e}"))?;
        let saldo_escalado = decimal_a_escalado(saldo_decimal)?;

        let id_generado = Uuid::new_v4().to_string();
        let creada_en = Utc::now().date_naive().to_string();

        // 2. Insertar evento inmutable CuentaCreada
        let datos_evento = json!({
            "id": id_generado,
            "nombre": req.nombre.trim(),
            "tipo": tipo_str,
            "moneda_codigo": req.moneda_codigo,
            "saldo_inicial": req.saldo_inicial,
            "creada_en": creada_en
        });

        bd::insertar_evento(&tx, "CuentaCreada", &datos_evento)
            .map_err(|e| format!("Error al registrar evento inmutable: {}", e))?;

        // 3. Insertar proyección en la tabla 'cuenta' con el saldo escalado (monto * 10^8)
        tx.execute(
            "
            INSERT INTO cuenta (id, nombre, tipo, moneda_codigo, saldo, creada_en)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                id_generado,
                req.nombre.trim(),
                tipo_str,
                req.moneda_codigo,
                saldo_escalado,
                creada_en,
            ],
        )
        .map_err(|e| format!("Error al guardar la cuenta en SQLite: {e}"))?;

        // 4. Confirmar transacción
        tx.commit()
            .map_err(|e| format!("Error al hacer commit en SQLite: {}", e))?;

        Ok(CuentaDTO {
            id: id_generado,
            nombre: req.nombre.trim().to_string(),
            tipo: req.tipo,
            moneda_codigo: req.moneda_codigo.clone(),
            saldo: escalado_a_decimal(saldo_escalado).to_string(),
            creada_en: Utc::now().to_rfc3339(),
            activa: true,
        })
    }

    fn obtener_cuentas(&self) -> Result<Vec<CuentaDTO>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, nombre, tipo, moneda_codigo, saldo, activa FROM cuenta")
            .map_err(|e| e.to_string())?;

        let iterador = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let nombre: String = row.get(1)?;
                let tipo_str: String = row.get(2)?;
                let moneda_codigo: String = row.get(3)?;
                let saldo_raw: i64 = row.get(4)?;
                let activa: i64 = row.get(5)?;

                let tipo: TipoCuentaDTO = serde_json::from_str(&format!("\"{}\"", tipo_str))
                    .unwrap_or(TipoCuentaDTO::Efectivo);

                let saldo = escalado_a_decimal(saldo_raw).to_string();

                Ok(CuentaDTO {
                    id,
                    nombre,
                    tipo,
                    moneda_codigo,
                    saldo,
                    creada_en: Utc::now().to_rfc3339(),
                    activa: activa != 0,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut lista = Vec::new();
        for cuenta in iterador {
            lista.push(cuenta.map_err(|e| e.to_string())?);
        }

        Ok(lista)
    }

    //Esta area es parte del trabajo de Ricardo, asi que luego le pregunto que tal
    fn obtener_cuenta_por_id(&self, id: &str) -> Result<CuentaDTO, String> {
        let cuentas = self.obtener_cuentas()?;
        cuentas
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("No se encontró ninguna cuenta con el ID: {}", id))
    }
}
