use aplicacion::puertos::{Reloj, RepositorioMonedas, RepositorioTasas};
use contrato::{
    error_validacion::ErrorValidacionDTO,
    tasa::{EliminarTasaRequest, RegistrarTasaRequest, TasaDTO},
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

#[tauri::command]
pub async fn eliminar_tasa(
    datos: EliminarTasaRequest,
    repo_tasas: State<'_, Box<dyn RepositorioTasas>>,
) -> Result<(), ErrorValidacionDTO> {
    aplicacion::casos_uso::eliminar_tasa::ejecutar(datos, repo_tasas.inner().as_ref()).await
}

#[tauri::command]
pub async fn listar_historial_tasas(
    fecha: String,
    repo_tasas: State<'_, Box<dyn RepositorioTasas>>,
) -> Result<Vec<TasaDTO>, String> {
    let fecha = chrono::NaiveDate::parse_from_str(&fecha, "%Y-%m-%d")
        .map_err(|_| "Formato de fecha inválido. Use AAAA-MM-DD.".to_string())?;

    aplicacion::casos_uso::listar_historial_tasas::ejecutar(fecha, repo_tasas.inner().as_ref())
        .await
}
