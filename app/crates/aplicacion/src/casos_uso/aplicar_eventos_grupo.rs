use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::puertos::{
    RepositorioCuentas, RepositorioMovimientos, RepositorioSincronizacion,
};
use contrato::EventoCambioDTO;
use dominio::movimiento::{Movimiento, TipoMovimiento};

pub fn aplicar_evento(
    evento: &EventoCambioDTO,
    nombre_grupo: Option<&str>,
    repo_sincronizacion: &dyn RepositorioSincronizacion,
    repo_movimientos: &dyn RepositorioMovimientos,
    repo_cuentas: &dyn RepositorioCuentas,
) -> Result<bool, String> {
    if repo_sincronizacion.evento_ya_aplicado(&evento.id_evento)? {
        return Ok(false);
    }

    let seleccion = repo_sincronizacion.obtener_cuenta_grupo(&evento.grupo_id)?;
    let Some(seleccion) = seleccion else {
        repo_movimientos.guardar_parte_grupo_pendiente(evento)?;
        return Ok(false);
    };

    let cuenta = repo_cuentas.obtener_cuenta_por_id(&seleccion.cuenta_id)?;

    if !cuenta.activa {
        repo_movimientos.guardar_parte_grupo_pendiente(evento)?;
        return Ok(false);
    }

    if !cuenta.activa || cuenta.moneda_codigo != evento.moneda || seleccion.moneda != evento.moneda {
        repo_movimientos.guardar_parte_grupo_pendiente(evento)?;
        return Ok(false);
    }

    let monto = Decimal::from_str(&evento.monto_parte)
        .map_err(|_| "La parte del gasto no es un monto válido.".to_string())?;
    let fecha = NaiveDate::parse_from_str(
        evento.fecha.get(..10).unwrap_or(&evento.fecha),
        "%Y-%m-%d",
    )
    .map_err(|_| "La fecha del gasto no es válida.".to_string())?;
    let categoria_id = Uuid::parse_str("00000000-0000-4000-8000-000000000001")
        .map_err(|_| "La categoría de gastos de grupo no es válida.".to_string())?;

    let movimiento = Movimiento {
        id: Uuid::new_v4(),
        cuenta_id: Uuid::parse_str(&cuenta.id)
            .map_err(|_| "La cuenta seleccionada no tiene un ID válido.".to_string())?,
        tipo: TipoMovimiento::Gasto,
        monto,
        fecha,
        categoria_id,
        nota: Some(format!(
            "{} · {}",
            nombre_grupo.unwrap_or(&evento.grupo_id),
            evento.concepto
        )),
    };

    repo_movimientos.guardar_aplicacion_gasto_grupo(&movimiento, evento)?;
    repo_sincronizacion.guardar_evento_aplicado(
        &evento.id_evento,
        &movimiento.id.to_string(),
    )?;
    Ok(true)
}
