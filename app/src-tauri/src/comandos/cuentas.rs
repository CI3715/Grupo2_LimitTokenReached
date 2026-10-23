use aplicacion::puertos::{RepositorioCuentas, RepositorioMonedas};
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::cuenta::CuentaDTO;
use contrato::error_validacion::ErrorValidacionDTO;
use tauri::State;

#[tauri::command]
pub async fn crear_cuenta(
    datos: CrearCuentaRequest,
    repo_cuentas: State<'_, Box<dyn RepositorioCuentas>>,
    repo_monedas: State<'_, Box<dyn RepositorioMonedas>>,
) -> Result<CuentaDTO, ErrorValidacionDTO> {
    aplicacion::casos_uso::crear_nueva_cuenta::ejecutar(
        datos,
        repo_cuentas.inner().as_ref(),
        repo_monedas.inner().as_ref(),
    )
    .await
}

#[tauri::command]
pub async fn listar_cuentas(
    repo: tauri::State<'_, Box<dyn aplicacion::puertos::RepositorioCuentas>>,
) -> Result<Vec<CuentaDTO>, String> {
    aplicacion::casos_uso::listar_cuentas::ejecutar(repo.inner().as_ref()).await
}
