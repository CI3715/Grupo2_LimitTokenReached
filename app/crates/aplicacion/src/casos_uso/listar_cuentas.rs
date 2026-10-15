use contrato::cuenta::CuentaDTO;
use crate::puertos::RepositorioCuentas;

/// Caso de uso para consultar el listado de cuentas financieras del usuario local
pub async fn ejecutar(repo: &dyn RepositorioCuentas) -> Result<Vec<CuentaDTO>, String> {
    repo.obtener_cuentas()
}