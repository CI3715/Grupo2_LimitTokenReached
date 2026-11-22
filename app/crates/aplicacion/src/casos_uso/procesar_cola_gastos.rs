use crate::puertos::{
    ClienteSocial, RepositorioColaGastos,
};

pub async fn ejecutar(
    cliente: &dyn ClienteSocial,
    token: String,
    repo_cola: &dyn RepositorioColaGastos,
) -> Result<(), crate::puertos::ErrorOperacion> {
    let operaciones = repo_cola
        .listar_en_orden()
        .map_err(|mensaje| crate::puertos::ErrorOperacion {
            status: None,
            mensaje,
        })?;

    for operacion in operaciones {
        match cliente
            .registrar_gasto(
                operacion.grupo_id,
                operacion.datos,
                operacion.idempotency_key.clone(),
                token.clone(),
            )
            .await
        {
            Ok(_) => {
                repo_cola
                    .eliminar(&operacion.idempotency_key)
                    .map_err(|mensaje| crate::puertos::ErrorOperacion {
                        status: None,
                        mensaje,
                    })?;
            }
            Err(error) => {
                if matches!(error.status, Some(400 | 409)) {
                    repo_cola
                        .marcar_rechazada(&operacion.idempotency_key, &error.mensaje)
                        .map_err(|mensaje| crate::puertos::ErrorOperacion {
                            status: None,
                            mensaje,
                        })?;
                } else {
                    return Err(error);
                }
            }
        }
    }

    Ok(())
}
