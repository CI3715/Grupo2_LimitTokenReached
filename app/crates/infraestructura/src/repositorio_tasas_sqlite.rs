use std::sync::Mutex;

use aplicacion::puertos::RepositorioTasas;
use chrono::NaiveDate;
use contrato::tasa::{RegistrarTasaRequest, TasaDTO};
use rusqlite::{Connection, params};
use serde_json::json;

pub struct RepositorioTasasSqlite {
    conexion: Mutex<Connection>,
}

impl RepositorioTasasSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

impl RepositorioTasas for RepositorioTasasSqlite {
    fn guardar_tasa(&self, request: &RegistrarTasaRequest) -> Result<TasaDTO, String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO tasas (moneda_entrada, moneda_salida, valor, fecha, fuente)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                request.moneda_origen,
                request.moneda_destino,
                request.valor,
                request.fecha,
                request.fuente
            ],
        )
        .map_err(|e| format!("No se pudo guardar la tasa: {e}"))?;

        let id = tx.last_insert_rowid().to_string();
        let datos_evento = json!({
            "tasa_id": id,
            "moneda_origen": request.moneda_origen,
            "moneda_destino": request.moneda_destino,
            "valor": request.valor,
            "fecha": request.fecha,
            "fuente": request.fuente
        });
        crate::bd::insertar_evento(&tx, "TasaRegistrada", &datos_evento)
            .map_err(|e| format!("No se pudo registrar el evento de tasa: {e}"))?;
        tx.commit()
            .map_err(|e| format!("No se pudo confirmar la tasa: {e}"))?;

        Ok(TasaDTO {
            id,
            moneda_origen: request.moneda_origen.clone(),
            moneda_destino: request.moneda_destino.clone(),
            valor: request.valor.clone(),
            fecha: request.fecha.clone(),
            fuente: request.fuente.clone(),
            eliminada: false,
        })
    }

    fn obtener_tasas_hasta(&self, fecha: NaiveDate) -> Result<Vec<TasaDTO>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT tasa_id, moneda_entrada, moneda_salida, valor, fecha, fuente
                 FROM tasas
                 WHERE fecha <= ?1
                 ORDER BY moneda_entrada, moneda_salida, fecha DESC, tasa_id DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([fecha.to_string()], |row| {
                Ok(TasaDTO {
                    id: row.get::<_, i64>(0)?.to_string(),
                    moneda_origen: row.get(1)?,
                    moneda_destino: row.get(2)?,
                    valor: row.get(3)?,
                    fecha: row.get(4)?,
                    fuente: row.get(5)?,
                    eliminada: false,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.map(|row| row.map_err(|e| e.to_string())).collect()
    }
}
