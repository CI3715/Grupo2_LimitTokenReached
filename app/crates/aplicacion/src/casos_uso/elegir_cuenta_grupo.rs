use crate::puertos::{
    RepositorioCuentas, RepositorioMovimientos, RepositorioSincronizacion,
};
use contrato::CuentaGrupoSeleccionDTO;

use super::aplicar_eventos_grupo;

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

    let mut aplicadas = 0;
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

        if aplicar_eventos_grupo::aplicar_evento(
            &evento,
            None,
            repo_sincronizacion,
            repo_movimientos,
            repo_cuentas,
        )? {
            repo_movimientos.eliminar_parte_grupo_pendiente(&evento.id_evento)?;
            aplicadas += 1;
        }
    }

    Ok(aplicadas)
}
