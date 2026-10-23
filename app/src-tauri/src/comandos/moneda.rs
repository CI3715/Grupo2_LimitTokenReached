use aplicacion::puertos::RepositorioMonedas;
use contrato::moneda::MonedaDTO;

#[tauri::command]
pub async fn listar_monedas(
    repo: tauri::State<'_, Box<dyn RepositorioMonedas>>,
) -> Result<Vec<MonedaDTO>, String> {
    aplicacion::casos_uso::listar_monedas::ejecutar(repo.inner().as_ref()).await
}
