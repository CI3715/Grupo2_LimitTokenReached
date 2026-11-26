use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use aplicacion::puertos::RepositorioEventos;
use dominio::libro::{EventoLibro, EventoRechazado, ResumenFusion, evaluar_fusion_lote};
use rusqlite::{Connection, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;

const ESCALA_MONTOS: i64 = 100_000_000; // 10^8

fn decimal_str_a_escalado(texto: &str) -> Result<i64, String> {
    let monto = texto
        .trim()
        .parse::<Decimal>()
        .map_err(|e| format!("Monto inválido '{texto}': {e}"))?;
    let escalado = monto
        .checked_mul(Decimal::from(ESCALA_MONTOS))
        .ok_or_else(|| "Desbordamiento al escalar monto".to_string())?;
    escalado
        .trunc()
        .to_i64()
        .ok_or_else(|| "Monto fuera de rango i64".to_string())
}

/// Normaliza recursivamente un `serde_json::Value` ordenando las claves de todos los objetos
/// para que dos JSON semánticamente iguales produzcan exactamente la misma cadena canónica.
pub fn canonizar_json(valor: &Value) -> Value {
    match valor {
        Value::Object(mapa) => {
            let mut ordenado = BTreeMap::new();
            for (k, v) in mapa {
                ordenado.insert(k.clone(), canonizar_json(v));
            }
            let mut salida = serde_json::Map::new();
            for (k, v) in ordenado {
                salida.insert(k, v);
            }
            Value::Object(salida)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonizar_json).collect()),
        otro => otro.clone(),
    }
}

pub fn canonizar_json_str(crudo: &str) -> Result<String, String> {
    let valor: Value =
        serde_json::from_str(crudo).map_err(|e| format!("JSON de evento inválido: {e}"))?;
    serde_json::to_string(&canonizar_json(&valor))
        .map_err(|e| format!("No se pudo serializar JSON canónico: {e}"))
}

pub struct RepositorioEventosSqlite {
    conexion: Mutex<Connection>,
}

impl RepositorioEventosSqlite {
    pub fn new(conexion: Connection) -> Self {
        Self {
            conexion: Mutex::new(conexion),
        }
    }

    fn leer_eventos_ordenados_tx(conn: &Connection) -> Result<Vec<EventoLibro>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, tipo, registrado_en, dispositivo, datos
                 FROM eventos
                 ORDER BY registrado_en ASC, dispositivo ASC, id ASC",
            )
            .map_err(|e| e.to_string())?;

        let filas = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut eventos = Vec::new();
        for fila in filas {
            let (id, tipo, registrado_en, dispositivo, datos_raw) =
                fila.map_err(|e| e.to_string())?;
            let datos_canonicos = canonizar_json_str(&datos_raw).unwrap_or(datos_raw);
            eventos.push(EventoLibro {
                id,
                tipo,
                registrado_en,
                dispositivo,
                datos_canonicos,
            });
        }
        Ok(eventos)
    }

    /// Reconstruye las tablas de proyección recorriendo todos los eventos
    /// en orden determinista `(registrado_en, dispositivo, id)` dentro de la transacción.
    fn reconstruir_en_tx(tx: &Transaction<'_>) -> Result<(), String> {
        let eventos = Self::leer_eventos_ordenados_tx(tx)?;

        // Limpiar tablas de proyección derivadas (la tabla `eventos` es inmutable y NO se toca)
        tx.execute("DELETE FROM movimiento_grupo", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM eventos_grupo_aplicados", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM ajuste", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM movimiento", [])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM cuenta", [])
            .map_err(|e| e.to_string())?;

        for ev in &eventos {
            let datos: Value = serde_json::from_str(&ev.datos_canonicos)
                .map_err(|e| format!("Evento {} corrupto: {e}", ev.id))?;

            match ev.tipo.as_str() {
                "CuentaCreada" => {
                    let id = datos["id"]
                        .as_str()
                        .unwrap_or(&ev.id);
                    let nombre = datos["nombre"]
                        .as_str()
                        .ok_or_else(|| format!("CuentaCreada {} sin nombre", ev.id))?;
                    let tipo = datos["tipo"].as_str().unwrap_or("Efectivo");
                    let moneda = datos["moneda_codigo"]
                        .as_str()
                        .ok_or_else(|| format!("CuentaCreada {} sin moneda_codigo", ev.id))?;
                    let saldo_str = datos["saldo_inicial"]
                        .as_str()
                        .or_else(|| datos["saldo"].as_str())
                        .unwrap_or("0");
                    let saldo_escalado = decimal_str_a_escalado(saldo_str)?;
                    let creada_en = datos["creada_en"]
                        .as_str()
                        .unwrap_or(&ev.registrado_en);

                    tx.execute(
                        "INSERT OR REPLACE INTO cuenta
                         (id, nombre, tipo, moneda_codigo, saldo, creada_en, activa)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                        params![id, nombre, tipo, moneda, saldo_escalado, creada_en],
                    )
                    .map_err(|e| format!("Error proyectando CuentaCreada {}: {e}", ev.id))?;
                }

                "CuentaRenombrada" => {
                    if let (Some(id), Some(nombre)) =
                        (datos["id"].as_str(), datos["nombre"].as_str())
                    {
                        tx.execute(
                            "UPDATE cuenta SET nombre = ?1 WHERE id = ?2",
                            params![nombre, id],
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }

                "CuentaArchivada" => {
                    if let Some(id) = datos["id"].as_str() {
                        tx.execute("UPDATE cuenta SET activa = 0 WHERE id = ?1", params![id])
                            .map_err(|e| e.to_string())?;
                    }
                }

                "MovimientoRegistrado" => {
                    let id = datos["id"]
                        .as_str()
                        .ok_or_else(|| format!("MovimientoRegistrado {} sin id", ev.id))?;
                    let cuenta_id = datos["cuenta_id"]
                        .as_str()
                        .ok_or_else(|| format!("MovimientoRegistrado {} sin cuenta_id", ev.id))?;
                    let tipo = datos["tipo"]
                        .as_str()
                        .ok_or_else(|| format!("MovimientoRegistrado {} sin tipo", ev.id))?;
                    let monto_str = datos["monto"]
                        .as_str()
                        .ok_or_else(|| format!("MovimientoRegistrado {} sin monto", ev.id))?;
                    let monto_escalado = decimal_str_a_escalado(monto_str)?;
                    let fecha = datos["fecha"]
                        .as_str()
                        .unwrap_or(&ev.registrado_en);
                    let categoria_id = datos["categoria_id"]
                        .as_str()
                        .unwrap_or("00000000-0000-0000-0000-000000000001");
                    let nota = datos["nota"].as_str();

                    tx.execute(
                        "INSERT INTO movimiento
                         (id, cuenta_id, tipo, monto, fecha, categoria_id, nota)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![id, cuenta_id, tipo, monto_escalado, fecha, categoria_id, nota],
                    )
                    .map_err(|e| format!("Error proyectando MovimientoRegistrado {}: {e}", ev.id))?;

                    let delta = if tipo.eq_ignore_ascii_case("Ingreso") {
                        monto_escalado
                    } else {
                        -monto_escalado
                    };

                    tx.execute(
                        "UPDATE cuenta SET saldo = saldo + ?1 WHERE id = ?2",
                        params![delta, cuenta_id],
                    )
                    .map_err(|e| format!("Error actualizando saldo en {}: {e}", ev.id))?;
                }

                "MovimientoCorregido" => {
                    let id = datos["id"].as_str().unwrap_or(&ev.id);
                    let movimiento_id = datos["movimiento_id"]
                        .as_str()
                        .ok_or_else(|| format!("MovimientoCorregido {} sin movimiento_id", ev.id))?;
                    let variacion_str = datos["variacion_monto"]
                        .as_str()
                        .ok_or_else(|| format!("MovimientoCorregido {} sin variacion_monto", ev.id))?;
                    let variacion_escalada = decimal_str_a_escalado(variacion_str)?;
                    let motivo = datos["motivo"].as_str().unwrap_or("");
                    let registrado_en = datos["registrado_en"]
                        .as_str()
                        .unwrap_or(&ev.registrado_en);

                    tx.execute(
                        "INSERT INTO ajuste
                         (id, movimiento_id, variacion_monto, motivo, registrado_en)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![id, movimiento_id, variacion_escalada, motivo, registrado_en],
                    )
                    .map_err(|e| format!("Error proyectando MovimientoCorregido {}: {e}", ev.id))?;

                    tx.execute(
                        "UPDATE cuenta
                         SET saldo = saldo + ?1
                         WHERE id = (SELECT cuenta_id FROM movimiento WHERE id = ?2)",
                        params![variacion_escalada, movimiento_id],
                    )
                    .map_err(|e| format!("Error aplicando ajuste {}: {e}", ev.id))?;
                }

                "GastoDeGrupoAplicado" => {
                    if let (Some(id_evento), Some(movimiento_id), Some(grupo_id), Some(gasto_id)) = (
                        datos["id_evento"].as_str(),
                        datos["movimiento_id"].as_str(),
                        datos["grupo_id"].as_str(),
                        datos["gasto_id"].as_str(),
                    ) {
                        tx.execute(
                            "INSERT OR IGNORE INTO eventos_grupo_aplicados
                             (id_evento, movimiento_id, aplicado_en)
                             VALUES (?1, ?2, ?3)",
                            params![id_evento, movimiento_id, ev.registrado_en],
                        )
                        .map_err(|e| e.to_string())?;

                        tx.execute(
                            "INSERT OR IGNORE INTO movimiento_grupo
                             (movimiento_id, id_evento, grupo_id, gasto_id, concepto_grupo, moneda)
                             SELECT m.id, ?1, ?2, ?3, m.nota, c.moneda_codigo
                             FROM movimiento m
                             JOIN cuenta c ON c.id = m.cuenta_id
                             WHERE m.id = ?4",
                            params![id_evento, grupo_id, gasto_id, movimiento_id],
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }

                // Eventos de catálogo/tasas o tipos futuros desconocidos se preservan
                // en la tabla `eventos` sin romper la reconstrucción.
                _ => {}
            }
        }

        Ok(())
    }
}

impl RepositorioEventos for RepositorioEventosSqlite {
    fn obtener_device_id(&self) -> Result<String, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        crate::bd::obtener_o_crear_dispositivo_id(&conn).map_err(|e| e.to_string())
    }

    fn listar_eventos_ordenados(&self) -> Result<Vec<EventoLibro>, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        Self::leer_eventos_ordenados_tx(&conn)
    }

    fn fusionar_lote_transaccional(
        &self,
        eventos: &[EventoLibro],
    ) -> Result<ResumenFusion, String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 1. Cargar eventos existentes indexados por `id`
        let existentes_lista = Self::leer_eventos_ordenados_tx(&tx)?;
        let mut existentes_mapa: HashMap<String, EventoLibro> = HashMap::with_capacity(existentes_lista.len());
        for ev in existentes_lista {
            existentes_mapa.insert(ev.id.clone(), ev);
        }

        // 2. Validar JSON de cada evento entrante y normalizar su payload canónico
        let mut entrantes_validos = Vec::with_capacity(eventos.len());
        let mut rechazos_previos = Vec::new();

        for ev in eventos {
            if ev.id.trim().is_empty() || ev.tipo.trim().is_empty() || ev.registrado_en.trim().is_empty() {
                rechazos_previos.push(EventoRechazado {
                    id: ev.id.clone(),
                    motivo: "Evento inválido: faltan campos obligatorios (id, tipo o registrado_en)".to_string(),
                });
                continue;
            }
            match canonizar_json_str(&ev.datos_canonicos) {
                Ok(canonico) => {
                    let mut normalizado = ev.clone();
                    normalizado.datos_canonicos = canonico;
                    entrantes_validos.push(normalizado);
                }
                Err(motivo) => {
                    rechazos_previos.push(EventoRechazado {
                        id: ev.id.clone(),
                        motivo,
                    });
                }
            }
        }

        // 3. Evaluar unión de conjuntos (idempotencia + rechazo por conflicto sin abortar el lote)
        let (nuevos, mut resumen) = evaluar_fusion_lote(&existentes_mapa, &entrantes_validos);
        resumen.total_recibidos = eventos.len();
        resumen.rechazados.extend(rechazos_previos);

        // 4. Insertar los nuevos eventos conservando su id, timestamp y dispositivo originales
        for ev in &nuevos {
            let valor_json: Value = serde_json::from_str(&ev.datos_canonicos)
                .map_err(|e| format!("Error interno serializando evento {}: {e}", ev.id))?;
            crate::bd::insertar_evento_con_id(
                &tx,
                &ev.id,
                &ev.tipo,
                &ev.registrado_en,
                &ev.dispositivo,
                &valor_json,
            )
            .map_err(|e| format!("Error al insertar evento {} en fusión: {e}", ev.id))?;
        }

        // 5. Reconstruir saldos y proyecciones en orden determinista si hubo cambios
        if !nuevos.is_empty() {
            Self::reconstruir_en_tx(&tx)?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(resumen)
    }

    fn reconstruir_proyecciones_desde_eventos(&self) -> Result<(), String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        Self::reconstruir_en_tx(&tx)?;
        tx.commit().map_err(|e| e.to_string())
    }
}