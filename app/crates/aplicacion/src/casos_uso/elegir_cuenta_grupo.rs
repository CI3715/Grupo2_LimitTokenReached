use crate::puertos::{
    AplicacionGastoPlan, LoteSincronizacionPlan, RepositorioCuentas,
    RepositorioMovimientos, RepositorioSincronizacion,
};
use contrato::CuentaGrupoSeleccionDTO;

use super::aplicar_eventos_grupo::{preparar_evento, ResultadoPreparacion};

pub fn ejecutar(
    seleccion: CuentaGrupoSeleccionDTO,
    moneda_grupo: &str,
    repo_sincronizacion: &dyn RepositorioSincronizacion,
    repo_movimientos: &dyn RepositorioMovimientos,
    repo_cuentas: &dyn RepositorioCuentas,
) -> Result<usize, String> {
    if seleccion.moneda != moneda_grupo {
        return Err("La cuenta elegida no tiene la moneda del grupo.".to_string());
    }

    let cuenta = repo_cuentas.obtener_cuenta_por_id(&seleccion.cuenta_id)?;
    if !cuenta.activa {
        return Err("La cuenta elegida no está activa.".to_string());
    }
    if cuenta.moneda_codigo != moneda_grupo {
        return Err("La cuenta elegida no tiene la moneda del grupo.".to_string());
    }

    repo_sincronizacion.guardar_cuenta_grupo(&seleccion)?;
    let pendientes =
        repo_movimientos.listar_partes_grupo_pendientes_por_grupo(&seleccion.grupo_id)?;

    let mut movimientos = Vec::new();
    for parte in pendientes {
        let evento = contrato::EventoCambioDTO {
            id_evento: parte.id_evento.clone(),
            tipo: "GastoDeGrupoRegistrado".to_string(),
            grupo_id: parte.grupo_id,
            gasto_id: parte.gasto_id,
            concepto: parte.concepto,
            fecha: parte.fecha,
            moneda: parte.moneda,
            monto_parte: parte.monto_parte,
            incluye_redondeo: parte.incluye_redondeo,
            posicion_cursor: parte.posicion_cursor,
        };

        match preparar_evento(
            &evento,
            None,
            repo_sincronizacion,
            repo_cuentas,
        )? {
            ResultadoPreparacion::Aplicar { movimiento, evento, efecto_saldo } => {
                movimientos.push(AplicacionGastoPlan {
                    movimiento,
                    evento,
                    efecto_saldo,
                });
            }
            ResultadoPreparacion::YaAplicado | ResultadoPreparacion::Pendiente(_) => {}
        }
    }

    let cursor = repo_sincronizacion.obtener_cursor()?;
    let plan = LoteSincronizacionPlan {
        nuevo_cursor: cursor,
        movimientos,
        pendientes: Vec::new(),
    };
    let aplicados = repo_sincronizacion.aplicar_lote_transaccional(&plan)?;

    let mut aplicadas = 0;
    for id_evento in aplicados {
        repo_movimientos.eliminar_parte_grupo_pendiente(&id_evento)?;
        aplicadas += 1;
    }

    for parte in repo_movimientos
        .listar_partes_grupo_pendientes_por_grupo(&seleccion.grupo_id)?
    {
        if repo_sincronizacion.evento_ya_aplicado(&parte.id_evento)? {
            repo_movimientos.eliminar_parte_grupo_pendiente(&parte.id_evento)?;
        }
    }

    Ok(aplicadas)
}
