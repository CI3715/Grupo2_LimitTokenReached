use aplicacion::puertos::RepositorioMonedas;
use contrato::{
    error_validacion::ErrorValidacionDTO,
    moneda::{CrearMonedaRequest, EliminarMonedaRequest, MonedaDTO},
};

#[tauri::command]
pub async fn eliminar_moneda(
    datos: EliminarMonedaRequest,
    repo_monedas: tauri::State<'_, Box<dyn RepositorioMonedas>>,
) -> Result<(), ErrorValidacionDTO> {
    aplicacion::casos_uso::eliminar_moneda::ejecutar(datos, repo_monedas.inner().as_ref()).await
}

#[tauri::command]
pub async fn listar_monedas(
    repo: tauri::State<'_, Box<dyn RepositorioMonedas>>,
) -> Result<Vec<MonedaDTO>, String> {
    aplicacion::casos_uso::listar_monedas::ejecutar(repo.inner().as_ref()).await
}

#[tauri::command]
pub async fn crear_moneda(
    datos: CrearMonedaRequest,
    repo_monedas: tauri::State<'_, Box<dyn RepositorioMonedas>>,
) -> Result<MonedaDTO, ErrorValidacionDTO> {
    aplicacion::casos_uso::crear_moneda::ejecutar(datos, repo_monedas.inner().as_ref()).await
}
