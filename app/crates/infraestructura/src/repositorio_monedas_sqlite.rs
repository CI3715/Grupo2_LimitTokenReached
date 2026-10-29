use rusqlite::Connection;
use std::sync::Mutex;

use aplicacion::puertos::RepositorioMonedas;
use contrato::moneda::{CrearMonedaRequest, MonedaDTO};

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
                let es_del_catalogo = matches!(codigo.as_str(), "VES" | "USD" | "EUR" | "USDT");

                Ok(MonedaDTO {
                    codigo,
                    nombre,
                    simbolo,
                    decimales,
                    es_del_catalogo,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut lista = Vec::new();
        for moneda in iterador {
            lista.push(moneda.map_err(|e| e.to_string())?);
        }

        Ok(lista)
    }
    fn guardar_moneda(
    &self,
    request: &CrearMonedaRequest,
) -> Result<MonedaDTO, String> {
    let mut conexion = self
        .conexion
        .lock()
        .map_err(|error| format!("No se pudo acceder a la base de datos: {error}"))?;

    let transaccion = conexion
        .transaction()
        .map_err(|error| format!("No se pudo iniciar la transacción: {error}"))?;

    transaccion
        .execute(
            "
            INSERT INTO moneda (codigo, nombre, simbolo, decimales)
            VALUES (?1, ?2, ?3, ?4)
            ",
            (
                &request.codigo,
                &request.nombre,
                &request.simbolo,
                request.decimales,
            ),
        )
        .map_err(|error| {
            format!(
                "No se pudo guardar la moneda '{}': {error}",
                request.codigo
            )
        })?;

    let datos_evento = serde_json::json!({
        "codigo": request.codigo,
        "nombre": request.nombre,
        "simbolo": request.simbolo,
        "decimales": request.decimales,
        "es_del_catalogo": false,
    });

    crate::bd::insertar_evento(&transaccion, "MonedaCreada", &datos_evento)
        .map_err(|error| format!("No se pudo registrar el evento de moneda: {error}"))?;

    transaccion
        .commit()
        .map_err(|error| format!("No se pudo confirmar la moneda: {error}"))?;

    Ok(MonedaDTO {
        codigo: request.codigo.clone(),
        nombre: request.nombre.clone(),
        simbolo: request.simbolo.clone(),
        decimales: request.decimales,
        es_del_catalogo: false,
    })
}

}
