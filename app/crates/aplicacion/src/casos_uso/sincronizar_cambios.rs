use crate::puertos::{
    ClienteSocial, RepositorioCuentas, RepositorioMovimientos, RepositorioSincronizacion,
};
use crate::casos_uso::aplicar_eventos_grupo;

pub async fn ejecutar(
    cliente: &dyn ClienteSocial,
    token: String,
    repo_sincronizacion: &dyn RepositorioSincronizacion,
    repo_movimientos: &dyn RepositorioMovimientos,
    repo_cuentas: &dyn RepositorioCuentas,
) -> Result<u64, crate::puertos::ErrorOperacion> {
    let mut cursor = repo_sincronizacion
        .obtener_cursor()
        .map_err(|mensaje| crate::puertos::ErrorOperacion {
            status: None,
            mensaje,
        })?;

    loop {
        let lote = cliente.obtener_cambios(cursor, token.clone()).await?;

        for evento in &lote.eventos {
            let nombre_grupo = match cliente
                .obtener_detalle_grupo(evento.grupo_id.clone(), token.clone())
                .await
            {
                Ok(detalle) => Some(detalle.nombre),
                Err(_) => None,
            };
            aplicar_eventos_grupo::aplicar_evento(
                evento,
                nombre_grupo.as_deref(),
                repo_sincronizacion,
                repo_movimientos,
                repo_cuentas,
            )
            .map_err(|mensaje| crate::puertos::ErrorOperacion {
                status: None,
                mensaje,
            })?;
        }

        cursor = lote.nuevo_cursor;
        repo_sincronizacion
            .guardar_cursor(cursor)
            .map_err(|mensaje| crate::puertos::ErrorOperacion {
                status: None,
                mensaje,
            })?;

        if !lote.hay_mas {
            break;
        }
    }

    repo_sincronizacion
        .guardar_ultima_sincronizacion(&chrono::Utc::now().to_rfc3339())
        .map_err(|mensaje| crate::puertos::ErrorOperacion {
            status: None,
            mensaje,
        })?;

    Ok(cursor)
}
