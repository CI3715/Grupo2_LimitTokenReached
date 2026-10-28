use std::str::FromStr;
use std::sync::Mutex;

use aplicacion::puertos::RepositorioPatrimonio;
use chrono::NaiveDate;
use contrato::patrimonio::SaldoCuentaDTO;
use rusqlite::Connection;
use rust_decimal::Decimal;

pub struct RepositorioPatrimonioSqlite {
    conexion: Mutex<Connection>,
}

impl RepositorioPatrimonioSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

impl RepositorioPatrimonio for RepositorioPatrimonioSqlite {
    fn obtener_saldos_a_fecha(&self, fecha: NaiveDate) -> Result<Vec<SaldoCuentaDTO>, String> {
        let mut conn = self.conexion.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let mut stmt = tx
            .prepare(
                "SELECT id, nombre, moneda_codigo, saldo, creada_en
                 FROM cuenta ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let cuentas = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        drop(stmt);

        let fecha_texto = fecha.to_string();
        let mut saldos = Vec::new();
        for (id, nombre, moneda, saldo_actual, creada_en) in cuentas {
            if creada_en > fecha_texto {
                continue;
            }
            saldos.push(calcular_saldo(
                &tx,
                &id,
                &nombre,
                &moneda,
                &saldo_actual,
                fecha,
            )?);
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(saldos)
    }
}

fn calcular_saldo(
    conn: &Connection,
    cuenta_id: &str,
    nombre: &str,
    moneda: &str,
    saldo_actual: &str,
    fecha_consulta: NaiveDate,
) -> Result<SaldoCuentaDTO, String> {
    let saldo_actual = Decimal::from_str(saldo_actual)
        .map_err(|e| format!("Saldo inválido en la cuenta {cuenta_id}: {e}"))?;
    let mut actividad_total = Decimal::ZERO;
    let mut actividad_hasta_fecha = Decimal::ZERO;

    let mut movimientos = conn
        .prepare("SELECT tipo, monto, fecha FROM movimiento WHERE cuenta_id = ?1")
        .map_err(|e| e.to_string())?;
    let filas = movimientos
        .query_map([cuenta_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for fila in filas {
        let (tipo, monto, fecha) = fila.map_err(|e| e.to_string())?;
        let efecto = efecto(&tipo, &monto)?;
        actividad_total += efecto;
        if parsear_fecha(&fecha)? <= fecha_consulta {
            actividad_hasta_fecha += efecto;
        }
    }

    let mut ajustes = conn
        .prepare("SELECT tipo_original, variacion_monto, fecha FROM ajuste WHERE cuenta_id = ?1")
        .map_err(|e| e.to_string())?;
    let filas = ajustes
        .query_map([cuenta_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for fila in filas {
        let (tipo, variacion, fecha) = fila.map_err(|e| e.to_string())?;
        let efecto = efecto(&tipo, &variacion)?;
        actividad_total += efecto;
        if parsear_fecha(&fecha)? <= fecha_consulta {
            actividad_hasta_fecha += efecto;
        }
    }

    let saldo_inicial = saldo_actual - actividad_total;
    Ok(SaldoCuentaDTO {
        cuenta_id: cuenta_id.to_string(),
        nombre_cuenta: nombre.to_string(),
        moneda: moneda.to_string(),
        saldo: (saldo_inicial + actividad_hasta_fecha)
            .normalize()
            .to_string(),
    })
}

fn efecto(tipo: &str, monto: &str) -> Result<Decimal, String> {
    let monto = Decimal::from_str(monto).map_err(|e| format!("Monto inválido: {e}"))?;
    match tipo {
        "Ingreso" => Ok(monto),
        "Gasto" => Ok(-monto),
        otro => Err(format!("Tipo de movimiento inválido: {otro}")),
    }
}

fn parsear_fecha(fecha: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(fecha, "%Y-%m-%d")
        .map_err(|e| format!("Fecha inválida almacenada en SQLite ({fecha}): {e}"))
}
