use crate::bd;
use serde_json::json;
use rusqlite::params;
use rusqlite::Connection;
use std::sync::Mutex;
use chrono::Utc;

use aplicacion::puertos::RepositorioCuentas;
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::cuenta::{CuentaDTO, TipoCuentaDTO};

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

        // 2. Insertar evento inmutable CuentaCreada
        let datos_evento = json!({
            "nombre": req.nombre.trim(),
            "tipo": tipo_str,
            "moneda_codigo": req.moneda_codigo,
            "saldo_inicial": req.saldo_inicial
        });

        bd::insertar_evento(&tx, "CuentaCreada", &datos_evento)
            .map_err(|e| format!("Error al registrar evento inmutable: {}", e))?;

        // 3. Insertar proyección en la tabla 'cuenta'
        tx.execute(
            "INSERT INTO cuenta (nombre, tipo, moneda_codigo, saldo) VALUES (?1, ?2, ?3, ?4)",
            params![req.nombre.trim(), tipo_str, req.moneda_codigo, req.saldo_inicial],
        )
        .map_err(|e| format!("Error al guardar la cuenta en SQLite: {}", e))?;

        let id_generado = tx.last_insert_rowid().to_string();

        // 4. CONFIRMAR LA TRANSACCIÓN EN DISCO
        tx.commit().map_err(|e| format!("Error al hacer commit en SQLite: {}", e))?;

        Ok(CuentaDTO {
            id: id_generado,
            nombre: req.nombre.trim().to_string(),
            tipo: req.tipo.clone(),
            moneda_codigo: req.moneda_codigo.clone(),
            saldo: req.saldo_inicial.clone(),
            creada_en: Utc::now().to_rfc3339(),
        })
    }

    fn obtener_cuentas(&self) -> Result<Vec<CuentaDTO>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, nombre, tipo, moneda_codigo, saldo FROM cuenta")
            .map_err(|e| e.to_string())?;

        let iterador = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let nombre: String = row.get(1)?;
                let tipo_str: String = row.get(2)?;
                let moneda_codigo: String = row.get(3)?;
                let saldo: f64 = row.get(4)?;

                let tipo: TipoCuentaDTO = serde_json::from_str(&format!("\"{}\"", tipo_str))
                    .unwrap_or(TipoCuentaDTO::Efectivo);

                Ok(CuentaDTO {
                    id: id.to_string(),
                    nombre,
                    tipo,
                    moneda_codigo,
                    saldo: saldo.to_string(),
                    creada_en: Utc::now().to_rfc3339(),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut lista = Vec::new();
        for cuenta in iterador {
            lista.push(cuenta.map_err(|e| e.to_string())?);
        }

        Ok(lista)
    }

    //Esta area es parte del trabajo de Ricardo, antes de hacer merge dile lo que hiciste y si deberias dejarlo o no
    fn obtener_cuenta_por_id(&self, id: &str) -> Result<CuentaDTO, String> {
        // Por ahora, podemos reutilizar tu método existente para no complicar el SQL
        let cuentas = self.obtener_cuentas()?;
        cuentas
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("No se encontró ninguna cuenta con el ID: {}", id))
    }



}