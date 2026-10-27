use serde::{Deserialize, Serialize};

/// Request para registrar una nueva tasa de cambio
/// Cumple con que el valor viaja como String para preservar precisión decimal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrarTasaRequest {
    pub moneda_origen: String,   // Código de la moneda origen 
    pub moneda_destino: String,  // Código de la moneda destino 
    pub valor: String,           // INV-03: String, no f64
    pub fecha: String,           // Formato ISO 8601 (ej. "2026-10-23")
    pub fuente: String,          // Texto libre ("BCV", "casa de cambio")
}

/// DTO para mostrar una tasa en el historial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasaDTO {
    pub id: String,
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: String,           // INV-03: String
    pub fecha: String,
    pub fuente: String,
    pub eliminada: bool,         // true si fue "eliminada" (evento TasaEliminada)
}

/// Request para eliminar una tasa (genera un evento TasaEliminada, INV-04)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliminarTasaRequest {
    pub tasa_id: String,
}