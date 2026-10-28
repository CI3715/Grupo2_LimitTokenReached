use aplicacion::puertos::{RepositorioMonedas, RepositorioPreferencias};
use contrato::{
    error_validacion::ErrorValidacionDTO,
    moneda::CambiarMonedaBaseRequest,
};
use tauri::State;

#[tauri::command]
pub async fn cambiar_moneda_base(
    datos: CambiarMonedaBaseRequest,
    repo_preferencias: State<'_, Box<dyn RepositorioPreferencias>>,
    repo_monedas: State<'_, Box<dyn RepositorioMonedas>>,
) -> Result<(), ErrorValidacionDTO> {
    aplicacion::casos_uso::cambiar_moneda_base::ejecutar(
        datos,
        repo_preferencias.inner().as_ref(),
        repo_monedas.inner().as_ref(),
    )
    .await
}
