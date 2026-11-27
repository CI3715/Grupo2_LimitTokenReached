use chrono::NaiveDate;
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::cuenta::CuentaDTO;
use contrato::moneda::{CrearMonedaRequest, MonedaDTO};
use contrato::movimiento::MovimientoDTO;
use contrato::patrimonio::SaldoCuentaDTO;
use contrato::tasa::{RegistrarTasaRequest, TasaDTO};
use dominio::categoria::Categoria;
use dominio::libro::{EventoLibro, EventoRechazado, ResumenFusion};
use dominio::movimiento::Movimiento;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AplicacionGastoPlan {
    pub movimiento: Movimiento,
    pub evento: EventoGrupoPreparado,
    pub efecto_saldo: rust_decimal::Decimal,
}

#[derive(Debug, Clone)]
pub struct PartePendientePlan {
    pub evento: EventoGrupoPreparado,
}

#[derive(Debug, Clone)]
pub struct EventoGrupoPreparado {
    pub id_evento: String,
    pub grupo_id: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,
    pub moneda: String,
    pub monto_parte: String,
    pub incluye_redondeo: bool,
    pub posicion_cursor: u64,
}

#[derive(Debug, Clone)]
pub struct LoteSincronizacionPlan {
    pub nuevo_cursor: u64,
    pub movimientos: Vec<AplicacionGastoPlan>,
    pub pendientes: Vec<PartePendientePlan>,
}

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
    fn obtener_detalle_grupo(
        &self,
        grupo_id: String,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<contrato::DetalleGrupoDTO, ErrorOperacion>> + Send + '_>>;
    fn registrar_gasto(
        &self,
        grupo_id: String,
        datos: contrato::RegistrarGastoRequestDTO,
        idempotency_key: String,
        token: String,
    ) -> Pin<Box<dyn Future<Output = Result<contrato::GastoCreadoDTO, ErrorOperacion>> + Send + '_>>;

    fn obtener_cambios(
        &self,
        desde: u64,
        token: String,
    ) -> Pin<
        Box<dyn Future<Output = Result<contrato::RespuestaCambiosDTO, ErrorOperacion>> + Send + '_>,
    >;
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
    fn movimiento_es_de_grupo(&self, movimiento_id: &Uuid) -> Result<bool, String>;
    fn listar_partes_grupo_pendientes(
        &self,
    ) -> Result<Vec<contrato::ParteGrupoPendienteDTO>, String>;
    fn listar_partes_grupo_pendientes_por_grupo(
        &self,
        grupo_id: &str,
    ) -> Result<Vec<contrato::ParteGrupoPendienteDTO>, String>;
    fn eliminar_parte_grupo_pendiente(&self, id_evento: &str) -> Result<(), String>;
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

pub trait RepositorioSincronizacion: Send + Sync {
    fn obtener_cursor(&self) -> Result<u64, String>;
    fn guardar_cursor(&self, cursor: u64) -> Result<(), String>;

    fn obtener_ultima_sincronizacion(&self) -> Result<Option<String>, String>;
    fn guardar_ultima_sincronizacion(&self, fecha: &str) -> Result<(), String>;

    fn evento_ya_aplicado(&self, id_evento: &str) -> Result<bool, String>;
    fn guardar_evento_aplicado(&self, id_evento: &str, movimiento_id: &str) -> Result<(), String>;

    fn obtener_cuenta_grupo(
        &self,
        grupo_id: &str,
    ) -> Result<Option<contrato::CuentaGrupoSeleccionDTO>, String>;

    fn guardar_cuenta_grupo(
        &self,
        seleccion: &contrato::CuentaGrupoSeleccionDTO,
    ) -> Result<(), String>;

    fn obtener_sesion_vencida(&self) -> Result<bool, String>;
    fn guardar_sesion_vencida(&self, vencida: bool) -> Result<(), String>;

    fn aplicar_lote_transaccional(
        &self,
        plan: &LoteSincronizacionPlan,
    ) -> Result<Vec<String>, String>;
}

pub trait RepositorioColaGastos: Send + Sync {
    fn guardar_pendiente(
        &self,
        operacion: &contrato::OperacionGastoOfflineDTO,
    ) -> Result<(), String>;

    fn listar_en_orden(&self) -> Result<Vec<contrato::OperacionGastoOfflineDTO>, String>;

    fn listar_estados_en_orden(&self) -> Result<Vec<contrato::GastoColaDTO>, String>;

    fn marcar_rechazada(&self, idempotency_key: &str, motivo: &str) -> Result<(), String>;

    fn eliminar(&self, idempotency_key: &str) -> Result<(), String>;
}

// =========================================================================
// Puertos Entrega 8: Motor de Eventos, Fusión, Serialización y Cifrado
// =========================================================================

/// Identificador en claro del archivo cifrado. No revela ningún dato del libro.
pub const MAGIA_RESPALDO_CIFRADO: &str = "CUENTASCLARAS-BACKUP";

/// Resultado de leer y validar un archivo de exportación en formato plano.
///
/// Un archivo con cualquier línea inválida se rechaza completo; los conflictos
/// por ID repetido se informan después, durante la fusión.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibroDeserializado {
    pub version: String,
    pub exportado_en: String,
    pub eventos: Vec<EventoLibro>,
}

/// Resultado de una sincronización del libro con cualquier transporte.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReporteSincronizacionLibro {
    pub eventos_enviados: usize,
    pub eventos_recibidos: usize,
    pub eventos_ya_existentes: usize,
    pub rechazados: Vec<EventoRechazado>,
    pub fecha_ultima_sincronizacion: String,
}

/// Resultado de comprobar la consistencia entre el historial de eventos y los saldos actuales.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReporteIntegridad {
    pub integro: bool,
    pub total_eventos_verificados: usize,
    pub cuentas_verificadas: usize,
    pub discrepancias: Vec<String>,
}

/// Lote cifrado leído del destino, con el dispositivo que lo publicó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoteRecibido {
    pub dispositivo: String,
    pub contenido: Vec<u8>,
}

/// Transporte de lotes cifrados entre dispositivos propios del usuario.
///
/// Cada adaptador (Git, carpeta compartida o falso en memoria) solo mueve bytes:
/// el cifrado, la fusión y el formato de los lotes quedan en la aplicación.
pub trait TransporteSincronizacion: Send + Sync {
    /// Trae los lotes publicados por todos los dispositivos.
    fn leer_lotes(&self) -> Result<Vec<LoteRecibido>, String>;

    /// Publica un lote nuevo en la carpeta del dispositivo indicado.
    /// El transporte nunca debe reescribir un lote ya existente.
    fn publicar_lote(&self, dispositivo: &str, contenido: &[u8]) -> Result<(), String>;
}

/// Almacén de secretos del sistema operativo.
///
/// La configuración no secreta y los secretos comparten claves estables, pero
/// solo las credenciales de Git y la frase de paso se escriben aquí. Ningún
/// valor se copia al libro, al destino ni a la exportación.
pub trait AlmacenSecretos: Send + Sync {
    fn guardar(&self, clave: &str, valor: &str) -> Result<(), String>;
    fn leer(&self, clave: &str) -> Result<Option<String>, String>;
    fn borrar(&self, clave: &str) -> Result<(), String>;
}

/// Puerto de almacenamiento del libro de eventos inmutables y reconstrucción de saldos.
pub trait RepositorioEventos: Send + Sync {
    /// Devuelve el UUID único de esta instalación.
    fn obtener_device_id(&self) -> Result<String, String>;

    /// Devuelve todos los eventos del libro ordenados por `(registrado_en, dispositivo, id)`.
    fn listar_eventos_ordenados(&self) -> Result<Vec<EventoLibro>, String>;

    /// Ejecuta la unión de conjuntos por `id` dentro de una única transacción SQLite:
    /// ignora idénticos, rechaza con motivo los conflictivos sin abortar el lote
    /// y reconstruye las proyecciones/saldos si hubo inserciones.
    fn fusionar_lote_transaccional(&self, eventos: &[EventoLibro])
    -> Result<ResumenFusion, String>;

    /// Recalcula todas las proyecciones (cuentas, movimientos, ajustes, tasas)
    /// recorriendo los eventos en orden determinista `(registrado_en, dispositivo, id)`.
    fn reconstruir_proyecciones_desde_eventos(&self) -> Result<(), String>;

    /// Comprueba en memoria que los eventos sean consistentes y coincidan con los saldos
    /// actuales en SQLite sin modificar ningún dato
    fn verificar_integridad(&self) -> Result<ReporteIntegridad, String>;
}

/// Puerto para lectura/escritura del formato plano (cabecera de 3 líneas + 1 evento por línea),
/// filtrando cualquier credencial o token de sesión.
pub trait SerializadorLibro: Send + Sync {
    fn serializar_plano(
        &self,
        device_id: &str,
        exportado_en: &str,
        eventos: &[EventoLibro],
    ) -> Result<Vec<u8>, String>;

    fn deserializar_plano(&self, contenido: &[u8]) -> Result<LibroDeserializado, String>;
}

/// Puerto criptográfico para derivación Argon2id y cifrado autenticado (AES-256-GCM / XChaCha20-Poly1305).
pub trait CifradorRespaldo: Send + Sync {
    fn cifrar(&self, datos_planos: &[u8], frase_paso: &str) -> Result<Vec<u8>, String>;
    fn descifrar(&self, paquete_cifrado: &[u8], frase_paso: &str) -> Result<Vec<u8>, String>;
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
