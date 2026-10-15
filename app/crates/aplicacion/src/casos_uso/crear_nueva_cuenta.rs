use contrato::cuenta::CuentaDTO;
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::error_validacion::ErrorValidacionDTO;
use crate::puertos::RepositorioCuentas;

pub async fn ejecutar(
    datos: CrearCuentaRequest,
    repo: &dyn RepositorioCuentas,
) -> Result<CuentaDTO, ErrorValidacionDTO> {
    // Validaciones de dominio previas
    if datos.nombre.trim().is_empty() {
        return Err(ErrorValidacionDTO {
            campo: "nombre".to_string(),
            mensaje: "El nombre de la cuenta no puede estar vacío.".to_string(),
        });
    }

    // Convertir el Result<CuentaDTO, String> a Result<CuentaDTO, ErrorValidacionDTO>
    repo.guardar_cuenta(&datos)
        .map_err(|err_msg| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje: err_msg,
        })
}