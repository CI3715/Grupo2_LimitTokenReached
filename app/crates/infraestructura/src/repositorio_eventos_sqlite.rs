use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use aplicacion::puertos::{ReporteIntegridad, RepositorioEventos};
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

        // Los catálogos sembrados por las migraciones (monedas, categorías y
        // tasas iniciales) no se borran: los eventos solo los complementan o
        // sobrescriben por identificador. Así una fusión no vacía el catálogo.
        // 1. Catálogos primero: monedas y categorías deben existir antes de
        //    proyectar tasas, cuentas y movimientos que las referencian.
        let mut eventos_tasa: Vec<(&EventoLibro, Value)> = Vec::new();
        for ev in &eventos {
            let datos: Value = serde_json::from_str(&ev.datos_canonicos)
                .map_err(|e| format!("Evento {} corrupto: {e}", ev.id))?;

            match ev.tipo.as_str() {
                "MonedaCreada" => {
                    if let (Some(codigo), Some(nombre), Some(simbolo), Some(decimales)) = (
                        datos["codigo"].as_str(),
                        datos["nombre"].as_str(),
                        datos["simbolo"].as_str(),
                        datos["decimales"].as_i64(),
                    ) {
                        tx.execute(
                            "INSERT OR REPLACE INTO moneda (codigo, nombre, simbolo, decimales)
                             VALUES (?1, ?2, ?3, ?4)",
                            params![codigo, nombre, simbolo, decimales],
                        )
                        .map_err(|e| format!("Error proyectando MonedaCreada {}: {e}", ev.id))?;
                    }
                }
                "MonedaEliminada" => {
                    if let Some(codigo) = datos["codigo"].as_str() {
                        tx.execute("DELETE FROM moneda WHERE codigo = ?1", params![codigo])
                            .map_err(|e| {
                                format!("Error proyectando MonedaEliminada {}: {e}", ev.id)
                            })?;
                    }
                }
                "CategoriaCreada" => {
                    if let (Some(id), Some(nombre), Some(tipo)) = (
                        datos["id"].as_str(),
                        datos["nombre"].as_str(),
                        datos["tipo"].as_str(),
                    ) {
                        tx.execute(
                            "INSERT OR REPLACE INTO categoria (id, nombre, tipo, archivada)
                             VALUES (?1, ?2, ?3, 0)",
                            params![id, nombre, tipo],
                        )
                        .map_err(|e| format!("Error proyectando CategoriaCreada {}: {e}", ev.id))?;
                    }
                }
                "TasaRegistrada" | "TasaEliminada" => {
                    eventos_tasa.push((ev, datos));
                }
                _ => {}
            }
        }

        for (ev, datos) in eventos_tasa {
            let tasa_id = datos["tasa_id"]
                .as_str()
                .ok_or_else(|| format!("{} {} sin tasa_id", ev.tipo, ev.id))?
                .parse::<i64>()
                .map_err(|_| format!("{} {} con tasa_id inválido", ev.tipo, ev.id))?;
            match ev.tipo.as_str() {
                "TasaRegistrada" => {
                    let entrada = datos["moneda_origen"]
                        .as_str()
                        .ok_or_else(|| format!("TasaRegistrada {} sin moneda_origen", ev.id))?;
                    let salida = datos["moneda_destino"]
                        .as_str()
                        .ok_or_else(|| format!("TasaRegistrada {} sin moneda_destino", ev.id))?;
                    let valor = datos["valor"]
                        .as_str()
                        .ok_or_else(|| format!("TasaRegistrada {} sin valor", ev.id))?;
                    let fecha = datos["fecha"]
                        .as_str()
                        .ok_or_else(|| format!("TasaRegistrada {} sin fecha", ev.id))?;
                    let fuente = datos["fuente"]
                        .as_str()
                        .ok_or_else(|| format!("TasaRegistrada {} sin fuente", ev.id))?;

                    tx.execute(
                        "INSERT OR REPLACE INTO tasas
                         (tasa_id, moneda_entrada, moneda_salida, valor, fecha, fuente, eliminada)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                        params![tasa_id, entrada, salida, valor, fecha, fuente],
                    )
                    .map_err(|e| format!("Error proyectando TasaRegistrada {}: {e}", ev.id))?;
                }
                "TasaEliminada" => {
                    tx.execute(
                        "UPDATE tasas SET eliminada = 1 WHERE tasa_id = ?1",
                        params![tasa_id],
                    )
                    .map_err(|e| format!("Error proyectando TasaEliminada {}: {e}", ev.id))?;
                }
                _ => {}
            }
        }

        // 2. Cuentas, movimientos, ajustes y gastos de grupo.
        for ev in &eventos {
            let datos: Value = serde_json::from_str(&ev.datos_canonicos)
                .map_err(|e| format!("Evento {} corrupto: {e}", ev.id))?;

            match ev.tipo.as_str() {
                "CuentaCreada" => {
                    let id = datos["id"].as_str().unwrap_or(&ev.id);
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
                    let creada_en = datos["creada_en"].as_str().unwrap_or(&ev.registrado_en);

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
                    let fecha = datos["fecha"].as_str().unwrap_or(&ev.registrado_en);
                    let categoria_id = datos["categoria_id"]
                        .as_str()
                        .unwrap_or("00000000-0000-0000-0000-000000000001");
                    let nota = datos["nota"].as_str();

                    tx.execute(
                        "INSERT INTO movimiento
                         (id, cuenta_id, tipo, monto, fecha, categoria_id, nota)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            id,
                            cuenta_id,
                            tipo,
                            monto_escalado,
                            fecha,
                            categoria_id,
                            nota
                        ],
                    )
                    .map_err(|e| {
                        format!("Error proyectando MovimientoRegistrado {}: {e}", ev.id)
                    })?;

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
                    let movimiento_id = datos["movimiento_id"].as_str().ok_or_else(|| {
                        format!("MovimientoCorregido {} sin movimiento_id", ev.id)
                    })?;
                    let variacion_str = datos["variacion_monto"].as_str().ok_or_else(|| {
                        format!("MovimientoCorregido {} sin variacion_monto", ev.id)
                    })?;
                    let variacion_escalada = decimal_str_a_escalado(variacion_str)?;
                    let motivo = datos["motivo"].as_str().unwrap_or("");
                    let registrado_en =
                        datos["registrado_en"].as_str().unwrap_or(&ev.registrado_en);

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

                // Los tipos futuros desconocidos se conservan en `eventos` sin
                // romper la reconstrucción de las proyecciones conocidas.
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
        let mut existentes_mapa: HashMap<String, EventoLibro> =
            HashMap::with_capacity(existentes_lista.len());
        for ev in existentes_lista {
            existentes_mapa.insert(ev.id.clone(), ev);
        }

        // 2. Validar JSON de cada evento entrante y normalizar su payload canónico
        let mut entrantes_validos = Vec::with_capacity(eventos.len());
        let mut rechazos_previos = Vec::new();

        for ev in eventos {
            if ev.id.trim().is_empty()
                || ev.tipo.trim().is_empty()
                || ev.registrado_en.trim().is_empty()
            {
                rechazos_previos.push(EventoRechazado {
                    id: ev.id.clone(),
                    motivo:
                        "Evento inválido: faltan campos obligatorios (id, tipo o registrado_en)"
                            .to_string(),
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

    fn verificar_integridad(&self) -> Result<ReporteIntegridad, String> {
        let conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let eventos = Self::leer_eventos_ordenados_tx(&conn)?;

        let mut saldos_esperados: HashMap<String, i64> = HashMap::new();
        let mut nombres_cuenta: HashMap<String, String> = HashMap::new();
        let mut cuenta_por_movimiento: HashMap<String, String> = HashMap::new();
        let mut discrepancias = Vec::new();

        for ev in &eventos {
            let datos: Value = match serde_json::from_str(&ev.datos_canonicos) {
                Ok(v) => v,
                Err(e) => {
                    discrepancias.push(format!("Evento {} con JSON inválido: {e}", ev.id));
                    continue;
                }
            };

            match ev.tipo.as_str() {
                "CuentaCreada" => {
                    let id = datos["id"].as_str().unwrap_or(&ev.id).to_string();
                    let nombre = datos["nombre"].as_str().unwrap_or(&id).to_string();
                    let saldo_str = datos["saldo_inicial"]
                        .as_str()
                        .or_else(|| datos["saldo"].as_str())
                        .unwrap_or("0");
                    match decimal_str_a_escalado(saldo_str) {
                        Ok(s) => {
                            saldos_esperados.insert(id.clone(), s);
                            nombres_cuenta.insert(id, nombre);
                        }
                        Err(e) => discrepancias.push(format!("Evento {}: {e}", ev.id)),
                    }
                }
                "MovimientoRegistrado" => {
                    if let (Some(mov_id), Some(cta_id), Some(tipo), Some(monto_str)) = (
                        datos["id"].as_str(),
                        datos["cuenta_id"].as_str(),
                        datos["tipo"].as_str(),
                        datos["monto"].as_str(),
                    ) {
                        match decimal_str_a_escalado(monto_str) {
                            Ok(monto) => {
                                let delta = if tipo.eq_ignore_ascii_case("Ingreso") {
                                    monto
                                } else {
                                    -monto
                                };
                                *saldos_esperados.entry(cta_id.to_string()).or_insert(0) += delta;
                                cuenta_por_movimiento
                                    .insert(mov_id.to_string(), cta_id.to_string());
                            }
                            Err(e) => discrepancias.push(format!("Evento {}: {e}", ev.id)),
                        }
                    } else {
                        discrepancias.push(format!(
                            "MovimientoRegistrado {} tiene campos incompletos",
                            ev.id
                        ));
                    }
                }
                "MovimientoCorregido" => {
                    if let (Some(mov_id), Some(var_str)) = (
                        datos["movimiento_id"].as_str(),
                        datos["variacion_monto"].as_str(),
                    ) {
                        match (
                            cuenta_por_movimiento.get(mov_id),
                            decimal_str_a_escalado(var_str),
                        ) {
                            (Some(cta_id), Ok(variacion)) => {
                                *saldos_esperados.entry(cta_id.clone()).or_insert(0) += variacion;
                            }
                            (None, _) => discrepancias.push(format!(
                                "Ajuste {} referencia movimiento inexistente {}",
                                ev.id, mov_id
                            )),
                            (_, Err(e)) => discrepancias.push(format!("Evento {}: {e}", ev.id)),
                        }
                    }
                }
                _ => {}
            }
        }

        let mut stmt = conn
            .prepare("SELECT id, nombre, saldo FROM cuenta")
            .map_err(|e| e.to_string())?;
        let filas = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut cuentas_bd: HashMap<String, (String, i64)> = HashMap::new();
        for fila in filas {
            let (id, nombre, saldo) = fila.map_err(|e| e.to_string())?;
            cuentas_bd.insert(id, (nombre, saldo));
        }

        for (id, saldo_evento) in &saldos_esperados {
            match cuentas_bd.get(id) {
                Some((nombre, saldo_actual)) if saldo_actual != saldo_evento => {
                    discrepancias.push(format!(
                        "Saldo inconsistente en cuenta '{}' ({}): en SQLite es {} pero los eventos suman {}",
                        nombre, id, saldo_actual, saldo_evento
                    ));
                }
                None => {
                    let nombre = nombres_cuenta.get(id).map(String::as_str).unwrap_or(id);
                    discrepancias.push(format!(
                        "La cuenta '{}' ({}) existe en los eventos pero falta en la proyección",
                        nombre, id
                    ));
                }
                _ => {}
            }
        }

        Ok(ReporteIntegridad {
            integro: discrepancias.is_empty(),
            total_eventos_verificados: eventos.len(),
            cuentas_verificadas: cuentas_bd.len(),
            discrepancias,
        })
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::bd::migrar;
    use rusqlite::Connection;
    use serde_json::json;

    #[test]
    fn reconstruye_tasas_registradas_y_eliminadas_desde_eventos() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrar(&mut conn).unwrap();

        crate::bd::insertar_evento_con_id(
            &conn,
            "ev-tasa-alta",
            "TasaRegistrada",
            "2026-11-20T10:00:00Z",
            "servidor",
            &json!({
                "tasa_id": "99",
                "moneda_origen": "VES",
                "moneda_destino": "USD",
                "valor": "887.96",
                "fecha": "2026-11-20",
                "fuente": "QA"
            }),
        )
        .unwrap();
        crate::bd::insertar_evento_con_id(
            &conn,
            "ev-tasa-baja",
            "TasaEliminada",
            "2026-11-21T10:00:00Z",
            "servidor",
            &json!({ "tasa_id": "99" }),
        )
        .unwrap();

        let repositorio = RepositorioEventosSqlite::new(conn);
        repositorio
            .reconstruir_proyecciones_desde_eventos()
            .unwrap();

        let conexion = repositorio.conexion.lock().unwrap();
        let (valor, eliminada): (String, i64) = conexion
            .query_row(
                "SELECT valor, eliminada FROM tasas WHERE tasa_id = 99",
                [],
                |fila| Ok((fila.get(0)?, fila.get(1)?)),
            )
            .unwrap();
        assert_eq!(valor, "887.96");
        assert_eq!(eliminada, 1);
    }
}
