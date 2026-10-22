use contrato::error_validacion::ErrorValidacionDTO;
use contrato::movimiento::MovimientoDTO;
use crate::puertos::RepositorioMovimientos;
use std::str::FromStr;
use uuid::Uuid;

pub async fn ejecutar(
    cuenta_id: String,
    repo_movimientos: &dyn RepositorioMovimientos,
) -> Result<Vec<MovimientoDTO>, ErrorValidacionDTO> {
    let cuenta_id = Uuid::from_str(&cuenta_id)
        .map_err(|_| ErrorValidacionDTO {
            campo: "cuenta_id".to_string(),
            mensaje: "ID de cuenta inválido".to_string(),
        })?;

    repo_movimientos
        .obtener_historial_de_cuenta(&cuenta_id)
        .map_err(|mensaje| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        })
}