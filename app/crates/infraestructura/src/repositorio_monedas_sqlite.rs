use rusqlite::Connection;
use std::sync::Mutex;

use aplicacion::puertos::RepositorioMonedas;
use contrato::moneda::MonedaDTO;

pub struct RepositorioMonedasSqlite {
    pub conexion: Mutex<Connection>,
}

impl RepositorioMonedasSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

impl RepositorioMonedas for RepositorioMonedasSqlite {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT codigo, nombre, simbolo, decimales FROM moneda")
            .map_err(|e| e.to_string())?;

        let iterador = stmt
            .query_map([], |row| {
                let codigo: String = row.get(0)?;
                let nombre: String = row.get(1)?;
                let simbolo: String = row.get(2)?;
                let decimales: u32 = row.get(3)?;

                Ok(MonedaDTO {
                    es_del_catalogo: matches!(codigo.as_str(), "VES" | "USD" | "EUR" | "USDT"),
                    codigo,
                    nombre,
                    simbolo,
                    decimales,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut lista = Vec::new();
        for moneda in iterador {
            lista.push(moneda.map_err(|e| e.to_string())?);
        }

        Ok(lista)
    }
}
