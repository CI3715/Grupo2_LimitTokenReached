use crate::puertos::RepositorioTasas;
use contrato::{error_validacion::ErrorValidacionDTO, tasa::EliminarTasaRequest};

pub async fn ejecutar(
    datos: EliminarTasaRequest,
    repo_tasas: &dyn RepositorioTasas,
) -> Result<(), ErrorValidacionDTO> {
    let tasa_id = datos.tasa_id.trim();

    if tasa_id.is_empty() {
        return Err(ErrorValidacionDTO {
            campo: "tasa_id".to_string(),
            mensaje: "El identificador de la tasa no puede estar vacío.".to_string(),
        });
    }

    repo_tasas
        .eliminar_tasa(tasa_id)
        .map_err(|mensaje| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        })
}
