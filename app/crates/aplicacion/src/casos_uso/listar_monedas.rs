use contrato::moneda::MonedaDTO;
use crate::puertos::RepositorioMonedas;

pub async fn ejecutar(repo: &dyn RepositorioMonedas) -> Result<Vec<MonedaDTO>, String> {
    repo.obtener_monedas()
}