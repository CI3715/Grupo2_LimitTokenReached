use rusqlite::Connection; 
use std::{path::Path, sync::Mutex};
use chrono::Utc;
use rusqlite::OptionalExtension;
use uuid::Uuid;

//Las migraciones deben aplicarse siempre EN ORDEN

const MIGRACIONES:  &[&str] = &[
    include_str!("../migrations/001_events.sql"), 
    include_str!("../migrations/002_bd_v1.sql"), 
    include_str!("../migrations/003_movements.sql")
]; 

pub struct BaseDatos(pub Mutex<Connection>); 


/// Devuelve el id de este dispositivo; lo crea en `meta` la primera vez.
fn id_dispositivo(conn: &Connection) -> Result<String, rusqlite::Error> {
    let existente: Option<String> = conn
        .query_row("SELECT valor FROM meta WHERE clave = 'dispositivo'", [], |f| f.get(0))
        .optional()?;

    if let Some(id) = existente {
        return Ok(id);
    }
    let nuevo = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO meta (clave, valor) VALUES ('dispositivo', ?1)",
        [&nuevo],
    )?;
    Ok(nuevo)
}

/// Registra un evento y devuelve su id.
pub fn insertar_evento(
    conn: &Connection,
    tipo: &str,
    datos: &serde_json::Value,
) -> Result<String, rusqlite::Error> {
    let id = Uuid::new_v4().to_string();
    let registrado_en = Utc::now().to_rfc3339();
    let dispositivo = id_dispositivo(conn)?;

    conn.execute(
        "INSERT INTO eventos (id, tipo, registrado_en, dispositivo, datos)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&id, tipo, &registrado_en, &dispositivo, datos.to_string()),
    )?;
    Ok(id)
}


pub fn abrir(ruta: &Path) -> Result<Connection, rusqlite::Error> {
    let mut conn = Connection::open(ruta)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrar(&mut conn)?;
    Ok(conn)
}

pub fn migrar(conn: &mut Connection) -> Result<(), rusqlite::Error> {
    let aplicadas: i64 = conn.pragma_query_value(None, "user_version", |f| f.get(0))?;

    for (i, sql) in MIGRACIONES.iter().enumerate().skip(aplicadas as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", (i + 1) as i64)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn migra_y_repetir_no_cambia_nada() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrar(&mut conn).unwrap();
        migrar(&mut conn).unwrap();

        let version: i64 = conn
            .pragma_query_value(None, "user_version", |f| f.get(0))
            .unwrap();
        assert_eq!(version, MIGRACIONES.len() as i64);

        let monedas: i64 = conn
            .query_row("SELECT COUNT(*) FROM moneda", [], |f| f.get(0))
            .unwrap();
        assert_eq!(monedas, 4); // VES, USD, EUR, USDT
    }

    #[test]
    fn los_eventos_son_inmutables() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrar(&mut conn).unwrap();
        conn.execute(
            "INSERT INTO eventos (id, tipo, registrado_en, dispositivo, datos)
             VALUES ('e1', 'CuentaCreada', '2026-10-15T00:00:00Z', 'd1', '{}')",
            [],
        )
        .unwrap();

        assert!(conn.execute("UPDATE eventos SET tipo = 'X'", []).is_err());
        assert!(conn.execute("DELETE FROM eventos", []).is_err());
    }
}