use std::sync::Mutex;

use aplicacion::puertos::RepositorioPreferencias;
use rusqlite::{Connection, OptionalExtension, params};

const MONEDA_BASE_POR_DEFECTO: &str = "USD";
const CLAVE_MONEDA_BASE: &str = "moneda_base";

pub struct RepositorioPreferenciasSqlite {
    conexion: Mutex<Connection>,
}

impl RepositorioPreferenciasSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

impl RepositorioPreferencias for RepositorioPreferenciasSqlite {
    fn obtener_moneda_base(&self) -> Result<String, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let moneda: Option<String> = conn
            .query_row(
                "SELECT valor FROM meta WHERE clave = ?1",
                [CLAVE_MONEDA_BASE],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(moneda.unwrap_or_else(|| MONEDA_BASE_POR_DEFECTO.to_string()))
    }

    fn guardar_moneda_base(&self, codigo: &str) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO meta (clave, valor) VALUES (?1, ?2)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![CLAVE_MONEDA_BASE, codigo],
        )
        .map_err(|e| format!("No se pudo guardar la moneda base: {e}"))?;
        Ok(())
    }
}
