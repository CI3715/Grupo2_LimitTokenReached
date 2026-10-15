use contrato::cuenta::{TipoCuentaDTO,CuentaDTO};
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::error_validacion::ErrorValidacionDTO;
use aplicacion::puertos::RepositorioCuentas;



#[tauri::command]
pub async fn crear_cuenta(
    datos: CrearCuentaRequest,
    repo: tauri::State<'_, Box<dyn RepositorioCuentas>>,
) -> Result<CuentaDTO, ErrorValidacionDTO> {
    aplicacion::casos_uso::crear_nueva_cuenta::ejecutar(datos, repo.inner().as_ref()).await
}

#[tauri::command]
pub async fn listar_cuentas(
    repo: tauri::State<'_, Box<dyn aplicacion::puertos::RepositorioCuentas>>,
) -> Result<Vec<CuentaDTO>, String> {
    aplicacion::casos_uso::listar_cuentas::ejecutar(repo.inner().as_ref()).await
}