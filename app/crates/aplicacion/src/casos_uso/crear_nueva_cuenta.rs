use contrato::cuenta::CuentaDTO;
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::error_validacion::ErrorValidacionDTO;
use crate::puertos::RepositorioCuentas;

pub async fn ejecutar(
    datos: CrearCuentaRequest,
    repo: &dyn RepositorioCuentas,
) -> Result<CuentaDTO, ErrorValidacionDTO> {
    if datos.nombre.trim().is_empty() {
        return Err("El nombre de la cuenta no puede estar vacío.".to_string());
    }
    repo.guardar_cuenta(&datos)
}