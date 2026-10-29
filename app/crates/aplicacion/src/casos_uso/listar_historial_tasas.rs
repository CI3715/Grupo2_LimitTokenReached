use crate::puertos::RepositorioTasas;
use chrono::NaiveDate;
use contrato::tasa::TasaDTO;

pub async fn ejecutar(
    fecha: NaiveDate,
    repo_tasas: &dyn RepositorioTasas,
) -> Result<Vec<TasaDTO>, String> {
    repo_tasas.obtener_historial_tasas_hasta(fecha)
}
