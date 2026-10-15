use serde::{Deserialize, Serialize};

/// Representa una moneda del catálogo base
/// Se devuelve al frontend para poblar el <select> del formulario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonedaDTO {
    pub codigo: String,     // "VES", "USD", "EUR", "USDT"
    pub nombre: String,     // "Bolívar", "Dólar estadounidense", etc.
    pub simbolo: String,    // "Bs.", "$", "€", "USDT"
    pub decimales: u32,     // 2 para todas las del catálogo actual
}