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

fn tasa_desde_fila(fila: &rusqlite::Row<'_>) -> Result<TasaDTO, rusqlite::Error> {
    Ok(TasaDTO {
        id: fila.get::<_, i64>(0)?.to_string(),
        moneda_origen: fila.get(1)?,
        moneda_destino: fila.get(2)?,
        valor: fila.get(3)?,
        fecha: fila.get(4)?,
        fuente: fila.get(5)?,
        eliminada: fila.get::<_, i64>(6)? != 0,
    })
}

impl RepositorioTasas for RepositorioTasasSqlite {
    fn guardar_tasa(&self, request: &RegistrarTasaRequest) -> Result<TasaDTO, String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO tasas (moneda_entrada, moneda_salida, valor, fecha, fuente, eliminada)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
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
        let conexion = self
            .conexion
            .lock()
            .map_err(|error| format!("No se pudo acceder a la base de datos: {error}"))?;

        let mut consulta = conexion
            .prepare(
                "
            SELECT
                tasa_id,
                moneda_entrada,
                moneda_salida,
                valor,
                fecha,
                fuente,
                eliminada
            FROM tasas
            WHERE fecha <= ?1
              AND eliminada = 0
            ORDER BY
                moneda_entrada ASC,
                moneda_salida ASC,
                fecha DESC,
                tasa_id DESC
            ",
            )
            .map_err(|error| format!("No se pudieron consultar las tasas: {error}"))?;

        let filas = consulta
            .query_map([fecha.to_string()], tasa_desde_fila)
            .map_err(|error| format!("No se pudieron leer las tasas: {error}"))?;

        filas
            .map(|fila| fila.map_err(|error| format!("No se pudo leer una tasa: {error}")))
            .collect()
    }

    fn obtener_historial_tasas_hasta(&self, fecha: NaiveDate) -> Result<Vec<TasaDTO>, String> {
        let conexion = self
            .conexion
            .lock()
            .map_err(|error| format!("No se pudo acceder a la base de datos: {error}"))?;

        let mut consulta = conexion
            .prepare(
                "
                SELECT
                    tasa_id,
                    moneda_entrada,
                    moneda_salida,
                    valor,
                    fecha,
                    fuente,
                    eliminada
                FROM tasas
                WHERE fecha <= ?1
                ORDER BY
                    moneda_entrada ASC,
                    moneda_salida ASC,
                    fecha DESC,
                    tasa_id DESC
                ",
            )
            .map_err(|error| format!("No se pudo consultar el historial de tasas: {error}"))?;

        let filas = consulta
            .query_map([fecha.to_string()], tasa_desde_fila)
            .map_err(|error| format!("No se pudo leer el historial de tasas: {error}"))?;

        filas
            .map(|fila| {
                fila.map_err(|error| format!("No se pudo leer una tasa del historial: {error}"))
            })
            .collect()
    }

    fn eliminar_tasa(&self, tasa_id: &str) -> Result<(), String> {
        let tasa_id: i64 = tasa_id
            .trim()
            .parse()
            .map_err(|_| "El identificador de la tasa no es válido.".to_string())?;

        let mut conexion = self
            .conexion
            .lock()
            .map_err(|error| format!("No se pudo acceder a la base de datos: {error}"))?;

        let transaccion = conexion
            .transaction()
            .map_err(|error| format!("No se pudo iniciar la transacción: {error}"))?;

        let filas_modificadas = transaccion
            .execute(
                "
            UPDATE tasas
            SET eliminada = 1
            WHERE tasa_id = ?1
              AND eliminada = 0
            ",
                [tasa_id],
            )
            .map_err(|error| format!("No se pudo eliminar la tasa: {error}"))?;

        if filas_modificadas == 0 {
            return Err(format!("No existe una tasa activa con el ID {tasa_id}."));
        }

        let datos_evento = json!({
            "tasa_id": tasa_id.to_string()
        });

        crate::bd::insertar_evento(&transaccion, "TasaEliminada", &datos_evento)
            .map_err(|error| format!("No se pudo registrar el evento de eliminación: {error}"))?;

        transaccion
            .commit()
            .map_err(|error| format!("No se pudo confirmar la eliminación: {error}"))?;

        Ok(())
    }
}
