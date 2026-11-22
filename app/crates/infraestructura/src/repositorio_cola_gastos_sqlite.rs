use std::sync::Mutex;

use aplicacion::puertos::RepositorioColaGastos;
use contrato::{GastoColaDTO, OperacionGastoOfflineDTO};
use rusqlite::{Connection, params};

pub struct RepositorioColaGastosSqlite {
    conexion: Mutex<Connection>,
}

impl RepositorioColaGastosSqlite {
    pub fn new(conexion: Connection) -> Self {
        Self {
            conexion: Mutex::new(conexion),
        }
    }
}

impl RepositorioColaGastos for RepositorioColaGastosSqlite {
    fn guardar_pendiente(
        &self,
        operacion: &OperacionGastoOfflineDTO,
    ) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        let datos_json =
            serde_json::to_string(&operacion.datos).map_err(|error| error.to_string())?;
        let siguiente_orden: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(orden), 0) + 1 FROM cola_gastos_grupo",
                [],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;

        conn.execute(
            "INSERT INTO cola_gastos_grupo
             (idempotency_key, grupo_id, moneda, datos_json, estado, motivo, orden, creado_en)
             VALUES (?1, ?2, ?3, ?4, 'pendiente', NULL, ?5, CURRENT_TIMESTAMP)
             ON CONFLICT(idempotency_key) DO NOTHING",
            params![
                operacion.idempotency_key,
                operacion.grupo_id,
                operacion.moneda,
                datos_json,
                siguiente_orden,
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn listar_en_orden(&self) -> Result<Vec<OperacionGastoOfflineDTO>, String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        let mut statement = conn
            .prepare(
                "SELECT idempotency_key, grupo_id, moneda, datos_json
                 FROM cola_gastos_grupo
                 WHERE estado = 'pendiente'
                 ORDER BY orden ASC",
            )
            .map_err(|error| error.to_string())?;

        let filas = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let mut operaciones = Vec::new();
        for fila in filas {
            let (idempotency_key, grupo_id, moneda, datos_json) =
                fila.map_err(|error| error.to_string())?;
            let datos = serde_json::from_str(&datos_json).map_err(|error| error.to_string())?;
            operaciones.push(OperacionGastoOfflineDTO {
                idempotency_key,
                grupo_id,
                moneda,
                datos,
            });
        }
        Ok(operaciones)
    }

    fn listar_estados_en_orden(&self) -> Result<Vec<GastoColaDTO>, String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        let mut statement = conn
            .prepare(
                "SELECT idempotency_key, grupo_id, moneda, datos_json, estado, motivo, orden
                 FROM cola_gastos_grupo
                 ORDER BY orden ASC",
            )
            .map_err(|error| error.to_string())?;

        let filas = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let mut gastos = Vec::new();
        for fila in filas {
            let (key, grupo, moneda, datos_json, estado, motivo, orden) =
                fila.map_err(|error| error.to_string())?;
            let datos: contrato::RegistrarGastoRequestDTO =
                serde_json::from_str(&datos_json).map_err(|error| error.to_string())?;
            gastos.push(GastoColaDTO {
                idempotency_key: key,
                grupo_id: grupo,
                concepto: datos.concepto,
                monto: datos.monto,
                moneda,
                estado,
                motivo,
                orden: orden as u64,
            });
        }
        Ok(gastos)
    }

    fn marcar_rechazada(
        &self,
        idempotency_key: &str,
        motivo: &str,
    ) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.execute(
            "UPDATE cola_gastos_grupo
             SET estado = 'rechazado', motivo = ?2
             WHERE idempotency_key = ?1",
            params![idempotency_key, motivo],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn eliminar(&self, idempotency_key: &str) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.execute(
            "DELETE FROM cola_gastos_grupo WHERE idempotency_key = ?1",
            params![idempotency_key],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}