use serde::{Deserialize, Serialize};

/// Representa una cuenta ya creada, tal como se muestra en la UI
/// El saldo se devuelve como string
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CuentaDTO {
    pub id: String,             // UUID del evento CuentaCreada
    pub nombre: String,         // "Cartera"
    pub tipo: TipoCuentaDTO,    // Efectivo, Banco, BilleteraDigital
    pub moneda_codigo: String,  // "VES", "USD", etc.
    pub saldo: String,          // "1250.50" (string, no float)
    pub creada_en: String,      // ISO 8601: "2026-10-15T12:00:00Z"
}

/// Tipos de cuenta permitidos
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TipoCuentaDTO {
    Efectivo,
    Banco,
    BilleteraDigital,
}