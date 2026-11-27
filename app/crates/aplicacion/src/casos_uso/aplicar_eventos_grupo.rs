use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::puertos::EventoGrupoPreparado;
use crate::puertos::{RepositorioCuentas, RepositorioSincronizacion};
use contrato::EventoCambioDTO;
use dominio::movimiento::{Movimiento, TipoMovimiento};

/// Espacio de nombres fijo de Cuentas Claras para derivar UUID v5 determinísticos.
const NAMESPACE_CUENTAS_CLARAS: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);

/// Genera un `movimiento.id` determinístico a partir del `id_evento` del servidor.
pub fn id_movimiento_desde_evento_servidor(id_evento_servidor: &str) -> Uuid {
    Uuid::new_v5(
        &NAMESPACE_CUENTAS_CLARAS,
        format!("movimiento-grupo:{id_evento_servidor}").as_bytes(),
    )
}

/// Genera un `event_id` determinístico para `MovimientoRegistrado` derivado de un grupo.
pub fn id_evento_movimiento_desde_servidor(id_evento_servidor: &str) -> String {
    Uuid::new_v5(
        &NAMESPACE_CUENTAS_CLARAS,
        format!("MovimientoRegistrado:{id_evento_servidor}").as_bytes(),
    )
    .to_string()
}

/// Genera un `event_id` determinístico para `GastoDeGrupoAplicado` a partir del `id_evento` del servidor.
pub fn id_evento_aplicado_desde_servidor(id_evento_servidor: &str) -> String {
    Uuid::new_v5(
        &NAMESPACE_CUENTAS_CLARAS,
        format!("GastoDeGrupoAplicado:{id_evento_servidor}").as_bytes(),
    )
    .to_string()
}

pub enum ResultadoPreparacion {
    YaAplicado,
    Pendiente(EventoGrupoPreparado),
    Aplicar {
        movimiento: Movimiento,
        evento: EventoGrupoPreparado,
        efecto_saldo: Decimal,
    },
}

pub fn evento_preparado(evento: &EventoCambioDTO) -> EventoGrupoPreparado {
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
    if !cuenta.activa || cuenta.moneda_codigo != evento.moneda || seleccion.moneda != evento.moneda
    {
        return Ok(ResultadoPreparacion::Pendiente(evento_preparado));
    }

    let monto = Decimal::from_str(&evento.monto_parte)
        .map_err(|_| "La parte del gasto no es un monto válido.".to_string())?;
    let fecha =
        NaiveDate::parse_from_str(evento.fecha.get(..10).unwrap_or(&evento.fecha), "%Y-%m-%d")
            .map_err(|_| "La fecha del gasto no es válida.".to_string())?;
    let categoria_id = Uuid::parse_str("00000000-0000-0000-0000-000000000007")
        .map_err(|_| "La categoría de gastos de grupo no es válida.".to_string())?;

    let movimiento = Movimiento {
        id: id_movimiento_desde_evento_servidor(&evento.id_evento),
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

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Congela el namespace, el material y los roles usados en la derivación.
    /// Si alguien cambia cualquiera de los tres, esta prueba falla antes de que
    /// la duplicación aparezca al sincronizar entre dispositivos.
    #[test]
    fn congela_los_uuid_v5_derivados_del_evento_del_servidor() {
        let id_evento = "evt-servidor-123";

        assert_eq!(
            id_movimiento_desde_evento_servidor(id_evento).to_string(),
            "5715e7cd-fd46-5882-9fdd-7b8a5787b929"
        );
        assert_eq!(
            id_evento_movimiento_desde_servidor(id_evento),
            "743a2b55-069c-5a75-a6ed-3abf71031ce4"
        );
        assert_eq!(
            id_evento_aplicado_desde_servidor(id_evento),
            "b11ac08a-bfd1-5832-9d75-5874619724bb"
        );
    }

    #[test]
    fn los_ids_derivados_son_v5_y_difieren_por_evento_y_por_rol() {
        let id = id_movimiento_desde_evento_servidor("evt-servidor-123");
        assert_eq!(id.to_string().as_bytes()[14], b'5');

        assert_ne!(
            id_evento_movimiento_desde_servidor("evt-servidor-123"),
            id_evento_movimiento_desde_servidor("evt-servidor-124")
        );
        assert_ne!(
            id_evento_movimiento_desde_servidor("evt-servidor-123"),
            id_evento_aplicado_desde_servidor("evt-servidor-123")
        );
    }
}
