use crate::puertos::{
    AplicacionGastoPlan, ClienteSocial, LoteSincronizacionPlan, PartePendientePlan,
    RepositorioCuentas, RepositorioSincronizacion,
};
use crate::casos_uso::aplicar_eventos_grupo::{
    preparar_evento, ResultadoPreparacion,
};
use std::collections::HashMap;

pub async fn ejecutar(
    cliente: &dyn ClienteSocial,
    token: String,
    repo_sincronizacion: &dyn RepositorioSincronizacion,
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
        let mut nombres_grupo = HashMap::new();
        let mut movimientos = Vec::new();
        let mut pendientes = Vec::new();
        for evento in &lote.eventos {
            if !nombres_grupo.contains_key(&evento.grupo_id) {
                let nombre_grupo = match cliente
                .obtener_detalle_grupo(evento.grupo_id.clone(), token.clone())
                .await
                {
                    Ok(detalle) => detalle.nombre,
                    Err(_) => evento.grupo_id.clone(),
                };
                nombres_grupo.insert(evento.grupo_id.clone(), nombre_grupo);
            }
            let nombre_grupo = nombres_grupo
                .get(&evento.grupo_id)
                .map(String::as_str);
            match preparar_evento(
                evento,
                nombre_grupo,
                repo_sincronizacion,
                repo_cuentas,
            )
            .map_err(|mensaje| crate::puertos::ErrorOperacion {
                status: None,
                mensaje,
            })? {
                ResultadoPreparacion::YaAplicado => {}
                ResultadoPreparacion::Pendiente(evento) => {
                    pendientes.push(PartePendientePlan { evento });
                }
                ResultadoPreparacion::Aplicar { movimiento, evento, efecto_saldo } => {
                    movimientos.push(AplicacionGastoPlan {
                        movimiento,
                        evento,
                        efecto_saldo,
                    });
                }
            }
        }

        cursor = lote.nuevo_cursor;
        let plan = LoteSincronizacionPlan {
            nuevo_cursor: cursor,
            movimientos,
            pendientes,
        };
        repo_sincronizacion
            .aplicar_lote_transaccional(&plan)
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
