use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Representa el registro de una nueva tasa de cambio.
/// Almacena el valor exacto con Decimal y fechas nativas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasaRegistrada {
    pub id: Uuid,
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: Decimal,
    pub fecha_aplicable: NaiveDate,
    pub fuente: String,
    pub registrada_en: DateTime<Utc>,
}

/// Representa la eliminación lógica (anulación) de una tasa registrada previamente.
/// En Event Sourcing no hacemos DELETE, emitimos este evento para invalidarla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasaEliminada {
    pub tasa_id: Uuid,
    pub fecha_eliminacion: DateTime<Utc>,
}

/// Representa la creación de una nueva moneda en el catálogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonedaCreada {
    pub codigo: String,  // Ej: "VES", "USD"
    pub nombre: String,  // Ej: "Bolívar", "Dólar"
    pub simbolo: String, // Ej: "Bs.", "$"
    pub decimales: u32,
    pub creada_en: DateTime<Utc>,
}

/// Representa un cambio en la configuración global de la moneda base del usuario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonedaBaseCambiada {
    pub nuevo_codigo: String,
    pub cambiada_en: DateTime<Utc>,
}

/// Enum unificado que agrupa todos los eventos del dominio patrimonial.
/// Esto permite que el Repositorio reciba un `EventoPatrimonio` genérico
/// y facilite la serialización/deserialización para la tabla de SQLite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventoPatrimonio {
    TasaRegistrada(TasaRegistrada),
    TasaEliminada(TasaEliminada),
    MonedaCreada(MonedaCreada),
    MonedaBaseCambiada(MonedaBaseCambiada),
}
