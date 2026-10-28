use aplicacion::puertos::{
    Reloj, RepositorioMonedas, RepositorioPatrimonio, RepositorioPreferencias, RepositorioTasas,
};
use contrato::{
    error_validacion::ErrorValidacionDTO,
    patrimonio::{ConsultarPatrimonioRequest, ConsultarPatrimonioResponse},
};
use tauri::State;

#[tauri::command]
pub async fn consultar_patrimonio(
    datos: ConsultarPatrimonioRequest,
    repo_patrimonio: State<'_, Box<dyn RepositorioPatrimonio>>,
    repo_tasas: State<'_, Box<dyn RepositorioTasas>>,
    repo_preferencias: State<'_, Box<dyn RepositorioPreferencias>>,
    repo_monedas: State<'_, Box<dyn RepositorioMonedas>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<ConsultarPatrimonioResponse, ErrorValidacionDTO> {
    aplicacion::casos_uso::consultar_patrimonio::ejecutar(
        datos,
        repo_patrimonio.inner().as_ref(),
        repo_tasas.inner().as_ref(),
        repo_preferencias.inner().as_ref(),
        repo_monedas.inner().as_ref(),
        reloj.inner().as_ref(),
    )
    .await
}
