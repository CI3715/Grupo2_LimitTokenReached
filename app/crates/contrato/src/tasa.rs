use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrarTasaRequest {
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: i64,                // INV-03: i64 (multiplicado por 10^8)
    pub fecha: String,
    pub fuente: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasaDTO {
    pub id: String,
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: i64,                // INV-03: i64
    pub fecha: String,
    pub fuente: String,
    pub eliminada: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliminarTasaRequest {
    pub tasa_id: String,
}