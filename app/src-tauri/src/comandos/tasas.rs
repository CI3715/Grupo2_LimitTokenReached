use aplicacion::puertos::{Reloj, RepositorioMonedas, RepositorioTasas};
use contrato::{
    error_validacion::ErrorValidacionDTO,
    tasa::{RegistrarTasaRequest, TasaDTO},
};
use tauri::State;

#[tauri::command]
pub async fn registrar_tasa(
    datos: RegistrarTasaRequest,
    repo_tasas: State<'_, Box<dyn RepositorioTasas>>,
    repo_monedas: State<'_, Box<dyn RepositorioMonedas>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<TasaDTO, ErrorValidacionDTO> {
    aplicacion::casos_uso::registrar_tasa::ejecutar(
        datos,
        repo_tasas.inner().as_ref(),
        repo_monedas.inner().as_ref(),
        reloj.inner().as_ref(),
    )
    .await
}
