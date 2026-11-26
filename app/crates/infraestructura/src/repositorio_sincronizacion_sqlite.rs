use aplicacion::puertos::RepositorioSincronizacion;
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde_json::json;
use std::sync::Mutex;

const ESCALA_MONTOS: i64 = 100_000_000; // 10^8

fn decimal_a_escalado(monto: Decimal) -> Result<i64, String> {
    let escalado = monto
        .checked_mul(Decimal::from(ESCALA_MONTOS))
        .ok_or_else(|| "Desbordamiento al escalar el monto".to_string())?;

    escalado
        .trunc()
        .to_i64()
        .ok_or_else(|| "El monto excede el rango permitido por SQLite (i64)".to_string())
}

fn escalado_a_decimal(valor: i64) -> Decimal {
    Decimal::new(valor, 8).normalize()
}

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
            .map(|cursor| {
                cursor
                    .parse()
                    .map_err(|error| format!("Cursor invalido: {error}"))
            })
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

    fn guardar_evento_aplicado(&self, id_evento: &str, movimiento_id: &str) -> Result<(), String> {
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

    fn obtener_sesion_vencida(&self) -> Result<bool, String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        let valor: Option<String> = conn
            .query_row(
                "SELECT valor FROM sincronizacion_grupo_meta
                 WHERE clave = 'sesion_vencida'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        Ok(valor.as_deref() == Some("true"))
    }

    fn guardar_sesion_vencida(&self, vencida: bool) -> Result<(), String> {
        let conn = self.conexion.lock().map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO sincronizacion_grupo_meta (clave, valor)
             VALUES ('sesion_vencida', ?1)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![if vencida { "true" } else { "false" }],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn aplicar_lote_transaccional(
        &self,
        plan: &aplicacion::puertos::LoteSincronizacionPlan,
    ) -> Result<Vec<String>, String> {
        let mut conn = self.conexion.lock().map_err(|error| error.to_string())?;
        let tx = conn.transaction().map_err(|error| error.to_string())?;
        let mut aplicados = Vec::new();

        for pendiente in &plan.pendientes {
            let evento = &pendiente.evento;
            let monto_parte_decimal = evento
                .monto_parte
                .trim()
                .parse::<Decimal>()
                .map_err(|error| format!("Monto de parte inválido: {error}"))?;
            let monto_parte_escalado = decimal_a_escalado(monto_parte_decimal)?;

            tx.execute(
                "INSERT INTO partes_grupo_pendientes
                 (id_evento, grupo_id, gasto_id, concepto, fecha, moneda,
                  monto_parte, incluye_redondeo, posicion_cursor, recibido_en)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)
                 ON CONFLICT(id_evento) DO NOTHING",
                params![
                    evento.id_evento,
                    evento.grupo_id,
                    evento.gasto_id,
                    evento.concepto,
                    evento.fecha,
                    evento.moneda,
                    monto_parte_escalado,
                    evento.incluye_redondeo as i64,
                    evento.posicion_cursor as i64,
                ],
            )
            .map_err(|error| error.to_string())?;
        }

        for aplicacion in &plan.movimientos {
            let movimiento = &aplicacion.movimiento;
            let evento = &aplicacion.evento;
            let ya_aplicado: Option<String> = tx
                .query_row(
                    "SELECT id_evento FROM eventos_grupo_aplicados WHERE id_evento = ?1",
                    params![evento.id_evento],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| error.to_string())?;

            if ya_aplicado.is_some() {
                continue;
            }

            let movimiento_id = movimiento.id.to_string();
            let cuenta_id = movimiento.cuenta_id.to_string();
            let categoria_id = movimiento.categoria_id.to_string();
            let concepto_movimiento = movimiento.nota.clone();
            let fecha = movimiento.fecha.to_string();
            let monto = movimiento.monto;
            let monto_escalado = decimal_a_escalado(monto)?;
            let efecto_saldo = aplicacion.efecto_saldo;

            tx.execute(
                "INSERT INTO movimiento
                 (id, cuenta_id, tipo, monto, fecha, categoria_id, nota)
                 VALUES (?1, ?2, 'Gasto', ?3, ?4, ?5, ?6)",
                params![
                    movimiento_id,
                    cuenta_id,
                    monto_escalado,
                    fecha,
                    categoria_id,
                    concepto_movimiento,
                ],
            )
            .map_err(|error| format!("No se pudo guardar el gasto de grupo: {error}"))?;

            let saldo_raw: i64 = tx
                .query_row(
                    "SELECT saldo FROM cuenta WHERE id = ?1",
                    params![cuenta_id],
                    |row| row.get(0),
                )
                .map_err(|error| format!("No se pudo obtener el saldo: {error}"))?;
            let saldo_actual = escalado_a_decimal(saldo_raw);
            let nuevo_saldo_escalado = decimal_a_escalado(saldo_actual + efecto_saldo)?;

            tx.execute(
                "UPDATE cuenta SET saldo = ?1 WHERE id = ?2",
                params![nuevo_saldo_escalado, cuenta_id],
            )
            .map_err(|error| format!("No se pudo actualizar el saldo: {error}"))?;

            let datos_movimiento = json!({
                "id": movimiento_id,
                "cuenta_id": cuenta_id,
                "tipo": "Gasto",
                "monto": monto.to_string(),
                "fecha": fecha,
                "categoria_id": categoria_id,
                "nota": concepto_movimiento,
            });
            let id_evento_movimiento =
                aplicacion::casos_uso::aplicar_eventos_grupo::id_evento_movimiento_desde_servidor(
                    &evento.id_evento,
                );
            crate::bd::insertar_evento_con_id(
                &tx,
                &id_evento_movimiento,
                "MovimientoRegistrado",
                &evento.fecha,
                "servidor",
                &datos_movimiento,
            )
            .map_err(|error| error.to_string())?;

            let datos_aplicado = json!({
                "id_evento": evento.id_evento,
                "movimiento_id": movimiento_id,
                "grupo_id": evento.grupo_id,
                "gasto_id": evento.gasto_id,
            });
            let id_evento_aplicado =
                aplicacion::casos_uso::aplicar_eventos_grupo::id_evento_aplicado_desde_servidor(
                    &evento.id_evento,
                );
            crate::bd::insertar_evento_con_id(
                &tx,
                &id_evento_aplicado,
                "GastoDeGrupoAplicado",
                &evento.fecha,
                "servidor",
                &datos_aplicado,
            )
            .map_err(|error| error.to_string())?;

            tx.execute(
                "INSERT INTO movimiento_grupo
                 (movimiento_id, id_evento, grupo_id, gasto_id, concepto_grupo, moneda)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    movimiento_id,
                    evento.id_evento,
                    evento.grupo_id,
                    evento.gasto_id,
                    concepto_movimiento,
                    evento.moneda,
                ],
            )
            .map_err(|error| error.to_string())?;

            tx.execute(
                "INSERT INTO eventos_grupo_aplicados
                 (id_evento, movimiento_id, aplicado_en)
                 VALUES (?1, ?2, CURRENT_TIMESTAMP)",
                params![evento.id_evento, movimiento_id],
            )
            .map_err(|error| error.to_string())?;

            aplicados.push(evento.id_evento.clone());
        }

        tx.execute(
            "INSERT INTO sincronizacion_grupo_meta (clave, valor) VALUES ('cursor', ?1)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![plan.nuevo_cursor.to_string()],
        )
        .map_err(|error| error.to_string())?;

        tx.commit().map_err(|error| error.to_string())?;
        Ok(aplicados)
    }
}