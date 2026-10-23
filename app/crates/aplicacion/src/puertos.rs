use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::cuenta::CuentaDTO;
use contrato::moneda::MonedaDTO;
use contrato::movimiento::MovimientoDTO;
use dominio::categoria::Categoria;
use dominio::movimiento::Movimiento;
use uuid::Uuid;

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
    fn obtener_cuenta_por_id(&self, id: &str) -> Result<CuentaDTO, String>;
}

pub trait RepositorioMonedas: Send + Sync {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String>;
}

pub trait RepositorioCategorias: Send + Sync {
    fn obtener_categorias(&self) -> Result<Vec<Categoria>, String>;

    fn guardar_categoria(
        &self,
        categoria: &Categoria,
    ) -> Result<contrato::categoria::CategoriaDTO, String>;
}

pub trait RepositorioMovimientos: Send + Sync {
    fn guardar_movimiento(&self, movimiento: &Movimiento) -> Result<MovimientoDTO, String>;
    fn guardar_ajuste(&self, ajuste: &dominio::movimiento::Ajuste)
    -> Result<MovimientoDTO, String>;
    fn obtener_movimiento_por_id(
        &self,
        id: &Uuid,
    ) -> Result<dominio::movimiento::Movimiento, String>;
    fn obtener_ajustes_de_movimiento(
        &self,
        original_id: &Uuid,
    ) -> Result<Vec<dominio::movimiento::Ajuste>, String>;
    fn obtener_historial_de_cuenta(&self, cuenta_id: &Uuid) -> Result<Vec<MovimientoDTO>, String>;
    fn obtener_categorias(&self) -> Result<Vec<Categoria>, String>;
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
