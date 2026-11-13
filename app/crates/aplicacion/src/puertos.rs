use chrono::NaiveDate;
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::cuenta::CuentaDTO;
use contrato::moneda::{CrearMonedaRequest, MonedaDTO};
use contrato::movimiento::MovimientoDTO;
use contrato::patrimonio::SaldoCuentaDTO;
use contrato::tasa::{RegistrarTasaRequest, TasaDTO};
use dominio::categoria::Categoria;
use dominio::movimiento::Movimiento;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

pub trait Reloj: Send + Sync {
    fn ahora(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorOperacion {
    pub status: Option<u16>,
    pub mensaje: String,
}

pub trait ClienteSocial: Send + Sync {
    fn buscar_usuario(
        &self,
        usuario: String,
        token: String,
    ) -> Pin<
        Box<dyn Future<Output = Result<contrato::UsuarioResumenDTO, ErrorOperacion>> + Send + '_>,
    >;
    fn invitar_amistad(
        &self,
        datos: contrato::CrearAmistadRequestDTO,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<contrato::AmistadCreadaDTO, ErrorOperacion>> + Send + '_>>;
    fn responder_invitacion_amistad(
        &self,
        invitacion_id: String,
        aceptar: bool,
        token: String,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<contrato::RespuestaInvitacionDTO, ErrorOperacion>>
                + Send
                + '_,
        >,
    >;
    fn listar_amistades(
        &self,
        token: String,
    ) -> Pin<
        Box<dyn Future<Output = Result<contrato::ListaAmistadesDTO, ErrorOperacion>> + Send + '_>,
    >;
    fn eliminar_amistad(
        &self,
        amistad_id: String,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), ErrorOperacion>> + Send + '_>>;
    fn eliminar_amistad_por_usuario(
        &self,
        nombre_usuario: String,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), ErrorOperacion>> + Send + '_>>;
    fn listar_grupos(
        &self,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<contrato::ListaGruposDTO, ErrorOperacion>> + Send + '_>>;
    fn crear_grupo(
        &self,
        datos: contrato::CrearGrupoRequestDTO,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<contrato::GrupoDTO, ErrorOperacion>> + Send + '_>>;
    fn invitar_miembro_grupo(
        &self,
        grupo_id: String,
        datos: contrato::InvitarMiembroGrupoRequestDTO,
        token: String,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<contrato::InvitacionGrupoCreadaDTO, ErrorOperacion>>
                + Send
                + '_,
        >,
    >;
    fn responder_invitacion_grupo(
        &self,
        grupo_id: String,
        aceptar: bool,
        token: String,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<contrato::RespuestaInvitacionGrupoDTO, ErrorOperacion>>
                + Send
                + '_,
        >,
    >;
    fn registrar_gasto(
        &self,
        grupo_id: String,
        datos: contrato::RegistrarGastoRequestDTO,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<contrato::GastoCreadoDTO, ErrorOperacion>> + Send + '_>>;
    fn obtener_detalle_grupo(
        &self,
        grupo_id: String,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<contrato::DetalleGrupoDTO, ErrorOperacion>> + Send + '_>>;
}

pub trait RepositorioCuentas: Send + Sync {
    fn guardar_cuenta(&self, request: &CrearCuentaRequest) -> Result<CuentaDTO, String>;
    fn obtener_cuentas(&self) -> Result<Vec<CuentaDTO>, String>;
    fn obtener_cuenta_por_id(&self, id: &str) -> Result<CuentaDTO, String>;
}

pub trait RepositorioMonedas: Send + Sync {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String>;

    fn guardar_moneda(&self, request: &CrearMonedaRequest) -> Result<MonedaDTO, String>;

    fn eliminar_moneda(&self, codigo: &str) -> Result<(), String>;
}

pub trait RepositorioTasas: Send + Sync {
    fn guardar_tasa(&self, request: &RegistrarTasaRequest) -> Result<TasaDTO, String>;
    /// Retorna únicamente tasas activas con fecha igual o anterior a la consulta.
    fn obtener_tasas_hasta(&self, fecha: NaiveDate) -> Result<Vec<TasaDTO>, String>;
    /// Retorna tasas activas y eliminadas con fecha igual o anterior a la consulta.
    fn obtener_historial_tasas_hasta(&self, fecha: NaiveDate) -> Result<Vec<TasaDTO>, String>;
    fn eliminar_tasa(&self, tasa_id: &str) -> Result<(), String>;
}

pub trait RepositorioPatrimonio: Send + Sync {
    fn obtener_saldos_a_fecha(&self, fecha: NaiveDate) -> Result<Vec<SaldoCuentaDTO>, String>;
}

pub trait RepositorioPreferencias: Send + Sync {
    fn obtener_moneda_base(&self) -> Result<String, String>;
    fn guardar_moneda_base(&self, codigo: &str) -> Result<(), String>;
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
