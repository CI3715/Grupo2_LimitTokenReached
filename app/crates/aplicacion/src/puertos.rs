use contrato::cuenta::CuentaDTO;
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::moneda::MonedaDTO;

pub trait Reloj: Send + Sync {
    fn ahora(&self) -> String;
}

// Puerto para abstracción de solicitudes HTTP salientes hacia el Servidor FastAPI
pub trait ClienteSocial: Send + Sync {
    fn hacer_ping(&self) -> impl std::future::Future<Output = Result<String, String>> + Send;
}



pub trait RepositorioCuentas: Send + Sync {
    fn guardar_cuenta(&self, request: &CrearCuentaRequest) -> Result<CuentaDTO, String>;
    fn obtener_cuentas(&self) -> Result<Vec<CuentaDTO>, String>;
}


pub trait RepositorioMonedas: Send + Sync {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String>;
}
// Adaptadores Fake

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RelojPrueba;

#[cfg(test)]
impl Reloj for RelojPrueba {
    fn ahora(&self) -> String {
        "2026-10-06T12:00:00Z".to_string() // Fecha fija e inmutable para tests
    }
}
