use crate::puertos::RepositorioMonedas;
use contrato::moneda::MonedaDTO;

pub async fn ejecutar(repo: &dyn RepositorioMonedas) -> Result<Vec<MonedaDTO>, String> {
    repo.obtener_monedas()
}
