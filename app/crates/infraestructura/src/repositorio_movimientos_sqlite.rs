use std::sync::Mutex;

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

use aplicacion::puertos::RepositorioMovimientos;
use contrato::movimiento::{MovimientoDTO, TipoMovimientoDTO};
use dominio::categoria::{Categoria, TipoCategoria};
use dominio::movimiento::{Ajuste, Movimiento, TipoMovimiento};

//funciones auxiliares

fn tipo_a_texto(tipo: TipoMovimiento) -> &'static str {
    match tipo {
        TipoMovimiento::Ingreso => "Ingreso",
        TipoMovimiento::Gasto => "Gasto",
    }
}

fn texto_a_tipo(valor: &str) -> Result<TipoMovimiento, String> {
    match valor {
        "Ingreso" => Ok(TipoMovimiento::Ingreso),
        "Gasto" => Ok(TipoMovimiento::Gasto),
        _ => Err(format!("Tipo de movimiento inválido: {valor}")),
    }
}

fn cargar_movimiento_desde(conn: &Connection, id: &Uuid) -> Result<Movimiento, String> {
    let fila = conn
        .query_row(
            "
            SELECT id, cuenta_id, tipo, monto, fecha, categoria_id, nota
            FROM movimiento
            WHERE id = ?1
            ",
            params![id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .map_err(|e| format!("Movimiento no encontrado: {e}"))?;

    Ok(Movimiento {
        id: Uuid::parse_str(&fila.0).map_err(|e| e.to_string())?,
        cuenta_id: Uuid::parse_str(&fila.1).map_err(|e| e.to_string())?,
        tipo: texto_a_tipo(&fila.2)?,
        monto: Decimal::from_str_exact(&fila.3).map_err(|e| e.to_string())?,
        fecha: NaiveDate::parse_from_str(&fila.4, "%Y-%m-%d").map_err(|e| e.to_string())?,
        categoria_id: Uuid::parse_str(&fila.5).map_err(|e| e.to_string())?,
        nota: fila.6,
    })
}

pub struct RepositorioMovimientosSqlite {
    pub conexion: Mutex<Connection>,
}

impl RepositorioMovimientosSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

impl RepositorioMovimientos for RepositorioMovimientosSqlite {
    fn guardar_movimiento(&self, movimiento: &Movimiento) -> Result<MovimientoDTO, String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let efecto = match movimiento.tipo {
            TipoMovimiento::Ingreso => movimiento.monto,
            TipoMovimiento::Gasto => -movimiento.monto,
        };

        tx.execute(
            "
        INSERT INTO movimiento
            (id, cuenta_id, tipo, monto, fecha, categoria_id, nota)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
            params![
                movimiento.id.to_string(),
                movimiento.cuenta_id.to_string(),
                tipo_a_texto(movimiento.tipo),
                movimiento.monto.to_string(),
                movimiento.fecha.to_string(),
                movimiento.categoria_id.to_string(),
                movimiento.nota,
            ],
        )
        .map_err(|e| format!("Error al guardar el movimiento: {e}"))?;

        let datos_evento = json!({
            "id": movimiento.id.to_string(),
            "cuenta_id": movimiento.cuenta_id.to_string(),
            "tipo": tipo_a_texto(movimiento.tipo),
            "monto": movimiento.monto.to_string(),
            "fecha": movimiento.fecha.to_string(),
            "categoria_id": movimiento.categoria_id.to_string(),
            "nota": movimiento.nota,
        });

        crate::bd::insertar_evento(&tx, "MovimientoRegistrado", &datos_evento)
            .map_err(|e| e.to_string())?;

        let saldo_actual: String = tx
            .query_row(
                "SELECT saldo FROM cuenta WHERE id = ?1",
                params![movimiento.cuenta_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| format!("No se pudo obtener el saldo: {e}"))?;

        let saldo =
            Decimal::from_str_exact(&saldo_actual).map_err(|e| format!("Saldo inválido: {e}"))?;

        let nuevo_saldo = saldo + efecto;

        tx.execute(
            "UPDATE cuenta SET saldo = ?1 WHERE id = ?2",
            params![nuevo_saldo.to_string(), movimiento.cuenta_id.to_string()],
        )
        .map_err(|e| format!("No se pudo actualizar el saldo: {e}"))?;

        tx.commit()
            .map_err(|e| format!("Error al confirmar el movimiento: {e}"))?;

        Ok(MovimientoDTO {
            id: movimiento.id.to_string(),
            cuenta_id: movimiento.cuenta_id.to_string(),
            tipo: match movimiento.tipo {
                TipoMovimiento::Ingreso => TipoMovimientoDTO::Ingreso,
                TipoMovimiento::Gasto => TipoMovimientoDTO::Gasto,
            },
            monto: movimiento.monto.to_string(),
            fecha: movimiento.fecha.to_string(),
            categoria_id: Some(movimiento.categoria_id.to_string()),
            nota: movimiento.nota.clone(),
            es_correccion: false,
            movimiento_original_id: None,
        })
    }

    fn guardar_ajuste(&self, ajuste: &Ajuste) -> Result<MovimientoDTO, String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let efecto = match ajuste.tipo_original {
            TipoMovimiento::Ingreso => ajuste.variacion_monto,
            TipoMovimiento::Gasto => -ajuste.variacion_monto,
        };

        tx.execute(
            "
            INSERT INTO ajuste
                (id, movimiento_original_id, cuenta_id, tipo_original, variacion_monto)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                ajuste.id.to_string(),
                ajuste.movimiento_original_id.to_string(),
                ajuste.cuenta_id.to_string(),
                tipo_a_texto(ajuste.tipo_original),
                ajuste.variacion_monto.to_string(),
            ],
        )
        .map_err(|e| format!("Error al guardar el ajuste: {e}"))?;

        let datos_evento = json!({
            "id": ajuste.id.to_string(),
            "movimiento_original_id": ajuste.movimiento_original_id.to_string(),
            "cuenta_id": ajuste.cuenta_id.to_string(),
            "tipo_original": tipo_a_texto(ajuste.tipo_original),
            "variacion_monto": ajuste.variacion_monto.to_string(),
        });

        crate::bd::insertar_evento(&tx, "MovimientoCorregido", &datos_evento)
            .map_err(|e| e.to_string())?;

        let saldo_actual: String = tx
            .query_row(
                "SELECT saldo FROM cuenta WHERE id = ?1",
                params![ajuste.cuenta_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| format!("No se pudo obtener el saldo: {e}"))?;

        let saldo =
            Decimal::from_str_exact(&saldo_actual).map_err(|e| format!("Saldo inválido: {e}"))?;

        let nuevo_saldo = saldo + efecto;

        tx.execute(
            "UPDATE cuenta SET saldo = ?1 WHERE id = ?2",
            params![nuevo_saldo.to_string(), ajuste.cuenta_id.to_string()],
        )
        .map_err(|e| format!("No se pudo actualizar el saldo: {e}"))?;

        let movimiento_original = cargar_movimiento_desde(&tx, &ajuste.movimiento_original_id)?;

        tx.commit()
            .map_err(|e| format!("Error al confirmar el ajuste: {e}"))?;

        Ok(MovimientoDTO {
            id: ajuste.id.to_string(),
            cuenta_id: ajuste.cuenta_id.to_string(),
            tipo: match ajuste.tipo_original {
                TipoMovimiento::Ingreso => TipoMovimientoDTO::Ingreso,
                TipoMovimiento::Gasto => TipoMovimientoDTO::Gasto,
            },
            monto: ajuste.variacion_monto.to_string(),
            fecha: movimiento_original.fecha.to_string(),
            categoria_id: Some(movimiento_original.categoria_id.to_string()),
            nota: None,
            es_correccion: true,
            movimiento_original_id: Some(ajuste.movimiento_original_id.to_string()),
        })
    }

    fn obtener_movimiento_por_id(&self, id: &Uuid) -> Result<Movimiento, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        cargar_movimiento_desde(&conn, id)
    }

    fn obtener_ajustes_de_movimiento(&self, original_id: &Uuid) -> Result<Vec<Ajuste>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "
            SELECT id, movimiento_original_id, cuenta_id,
                   tipo_original, variacion_monto
            FROM ajuste
            WHERE movimiento_original_id = ?1
            ORDER BY rowid
            ",
            )
            .map_err(|e| e.to_string())?;

        let filas = stmt
            .query_map(params![original_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        filas
            .map(|fila| {
                let (id, movimiento_original_id, cuenta_id, tipo_original, variacion_monto) =
                    fila.map_err(|e| e.to_string())?;

                Ok(Ajuste {
                    id: Uuid::parse_str(&id).map_err(|e| e.to_string())?,
                    movimiento_original_id: Uuid::parse_str(&movimiento_original_id)
                        .map_err(|e| e.to_string())?,
                    cuenta_id: Uuid::parse_str(&cuenta_id).map_err(|e| e.to_string())?,
                    tipo_original: texto_a_tipo(&tipo_original)?,
                    variacion_monto: Decimal::from_str_exact(&variacion_monto)
                        .map_err(|e| e.to_string())?,
                })
            })
            .collect()
    }

    fn obtener_categorias(&self) -> Result<Vec<Categoria>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "
            SELECT id, nombre, tipo
            FROM categoria
            WHERE archivada = 0
            ",
            )
            .map_err(|e| e.to_string())?;

        let filas = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        filas
            .map(|fila| {
                let (id, nombre, tipo) = fila.map_err(|e| e.to_string())?;

                Ok(Categoria {
                    id: Uuid::parse_str(&id).map_err(|e| e.to_string())?,
                    nombre,
                    tipo: match tipo.as_str() {
                        "Ingreso" => TipoCategoria::Ingreso,
                        "Gasto" => TipoCategoria::Gasto,
                        otro => return Err(format!("Tipo de categoría inválido: {otro}")),
                    },
                })
            })
            .collect()
    }

    fn obtener_historial_de_cuenta(&self, cuenta_id: &Uuid) -> Result<Vec<MovimientoDTO>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "
            SELECT
                id,
                cuenta_id,
                tipo,
                monto,
                fecha,
                categoria_id,
                nota,
                es_correccion,
                movimiento_original_id
            FROM (
                SELECT
                    m.id AS id,
                    m.cuenta_id AS cuenta_id,
                    m.tipo AS tipo,
                    m.monto AS monto,
                    m.fecha AS fecha,
                    m.categoria_id AS categoria_id,
                    m.nota AS nota,
                    0 AS es_correccion,
                    NULL AS movimiento_original_id,
                    0 AS orden_evento
                FROM movimiento m
                WHERE m.cuenta_id = ?1

                UNION ALL

                SELECT
                    a.id AS id,
                    a.cuenta_id AS cuenta_id,
                    a.tipo_original AS tipo,
                    a.variacion_monto AS monto,
                    m.fecha AS fecha,
                    m.categoria_id AS categoria_id,
                    NULL AS nota,
                    1 AS es_correccion,
                    a.movimiento_original_id AS movimiento_original_id,
                    1 AS orden_evento
                FROM ajuste a
                INNER JOIN movimiento m
                    ON m.id = a.movimiento_original_id
                WHERE a.cuenta_id = ?1
            )
            ORDER BY fecha DESC, orden_evento ASC, id ASC
            ",
            )
            .map_err(|e| format!("No se pudo preparar el historial: {e}"))?;

        let filas = stmt
            .query_map(params![cuenta_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, i64>(7)? != 0,
                    row.get::<_, Option<String>>(8)?,
                ))
            })
            .map_err(|e| format!("No se pudo consultar el historial: {e}"))?;

        let filas = filas
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("No se pudieron leer las filas del historial: {e}"))?;

        filas
            .into_iter()
            .map(
                |(
                    id,
                    cuenta_id,
                    tipo,
                    monto,
                    fecha,
                    categoria_id,
                    nota,
                    es_correccion,
                    movimiento_original_id,
                )| {
                    let tipo = match tipo.as_str() {
                        "Ingreso" => TipoMovimientoDTO::Ingreso,
                        "Gasto" => TipoMovimientoDTO::Gasto,
                        otro => {
                            return Err(format!(
                                "Tipo de movimiento inválido en el historial: {otro}"
                            ));
                        }
                    };

                    Ok(MovimientoDTO {
                        id,
                        cuenta_id,
                        tipo,
                        monto,
                        fecha,
                        categoria_id: Some(categoria_id),
                        nota,
                        es_correccion,
                        movimiento_original_id,
                    })
                },
            )
            .collect()
    }
}
