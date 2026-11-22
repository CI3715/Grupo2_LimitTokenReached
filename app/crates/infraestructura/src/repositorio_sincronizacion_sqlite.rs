use std::sync::Mutex;

use aplicacion::puertos::RepositorioSincronizacion;
use rusqlite::{Connection, OptionalExtension, params};

pub struct RepositorioSincronizacionSqlite {
    conexion: Mutex<Connection>,
}

impl RepositorioSincronizacionSqlite {
    pub fn new(conexion: Connection) -> Self {
        Self {
            conexion: Mutex::new(conexion),
        }
    }

}

impl RepositorioSincronizacion for RepositorioSincronizacionSqlite {
    fn obtener_cursor(&self) -> Result<u64, String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        let valor: Option<String> = conn
            .query_row(
                "SELECT valor FROM sincronizacion_grupo_meta WHERE clave = 'cursor'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        valor
            .map(|cursor| cursor.parse().map_err(|error| format!("Cursor invalido: {error}")))
            .transpose()
            .map(|cursor| cursor.unwrap_or(0))
    }

    fn guardar_cursor(&self, cursor: u64) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO sincronizacion_grupo_meta (clave, valor) VALUES ('cursor', ?1)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![cursor.to_string()],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn obtener_ultima_sincronizacion(&self) -> Result<Option<String>, String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.query_row(
            "SELECT valor FROM sincronizacion_grupo_meta
             WHERE clave = 'ultima_sincronizacion'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())
    }

    fn guardar_ultima_sincronizacion(&self, fecha: &str) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO sincronizacion_grupo_meta (clave, valor)
             VALUES ('ultima_sincronizacion', ?1)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![fecha],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn evento_ya_aplicado(&self, id_evento: &str) -> Result<bool, String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        let existente: Option<String> = conn
            .query_row(
                "SELECT id_evento FROM eventos_grupo_aplicados WHERE id_evento = ?1",
                params![id_evento],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        Ok(existente.is_some())
    }

    fn guardar_evento_aplicado(
        &self,
        id_evento: &str,
        movimiento_id: &str,
    ) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO eventos_grupo_aplicados
             (id_evento, movimiento_id, aplicado_en)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(id_evento) DO NOTHING",
            params![id_evento, movimiento_id],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn obtener_cuenta_grupo(
        &self,
        grupo_id: &str,
    ) -> Result<Option<contrato::CuentaGrupoSeleccionDTO>, String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.query_row(
            "SELECT grupo_id, cuenta_id, moneda
             FROM cuenta_por_grupo
             WHERE grupo_id = ?1",
            params![grupo_id],
            |row| {
                Ok(contrato::CuentaGrupoSeleccionDTO {
                    grupo_id: row.get(0)?,
                    cuenta_id: row.get(1)?,
                    moneda: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(|error| error.to_string())
    }

    fn guardar_cuenta_grupo(
        &self,
        seleccion: &contrato::CuentaGrupoSeleccionDTO,
    ) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO cuenta_por_grupo
             (grupo_id, cuenta_id, moneda, actualizada_en)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(grupo_id) DO UPDATE SET
                cuenta_id = excluded.cuenta_id,
                moneda = excluded.moneda,
                actualizada_en = excluded.actualizada_en",
            params![seleccion.grupo_id, seleccion.cuenta_id, seleccion.moneda],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}