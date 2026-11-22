use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::puertos::{
    RepositorioCuentas, RepositorioSincronizacion,
};
use contrato::EventoCambioDTO;
use dominio::movimiento::{Movimiento, TipoMovimiento};
use crate::puertos::EventoGrupoPreparado;

pub enum ResultadoPreparacion {
    YaAplicado,
    Pendiente(EventoGrupoPreparado),
    Aplicar {
        movimiento: Movimiento,
        evento: EventoGrupoPreparado,
        efecto_saldo: Decimal,
    },
}

pub fn evento_preparado(
    evento: &EventoCambioDTO,
) -> EventoGrupoPreparado {
    EventoGrupoPreparado {
        id_evento: evento.id_evento.clone(),
        grupo_id: evento.grupo_id.clone(),
        gasto_id: evento.gasto_id.clone(),
        concepto: evento.concepto.clone(),
        fecha: evento.fecha.clone(),
        moneda: evento.moneda.clone(),
        monto_parte: evento.monto_parte.clone(),
        incluye_redondeo: evento.incluye_redondeo,
        posicion_cursor: evento.posicion_cursor,
    }
}

pub fn preparar_evento(
    evento: &EventoCambioDTO,
    nombre_grupo: Option<&str>,
    repo_sincronizacion: &dyn RepositorioSincronizacion,
    repo_cuentas: &dyn RepositorioCuentas,
) -> Result<ResultadoPreparacion, String> {
    if repo_sincronizacion.evento_ya_aplicado(&evento.id_evento)? {
        return Ok(ResultadoPreparacion::YaAplicado);
    }
    let evento_preparado = evento_preparado(evento);

    let seleccion = repo_sincronizacion.obtener_cuenta_grupo(&evento.grupo_id)?;
    let Some(seleccion) = seleccion else {
        return Ok(ResultadoPreparacion::Pendiente(evento_preparado));
    };

    let cuenta = repo_cuentas.obtener_cuenta_por_id(&seleccion.cuenta_id)?;
    if !cuenta.activa
        || cuenta.moneda_codigo != evento.moneda
        || seleccion.moneda != evento.moneda
    {
        return Ok(ResultadoPreparacion::Pendiente(evento_preparado));
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

    Ok(ResultadoPreparacion::Aplicar {
        movimiento,
        evento: evento_preparado,
        efecto_saldo: -monto,
    })
}

