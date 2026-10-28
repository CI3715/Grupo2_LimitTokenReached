use serde::{Deserialize, Serialize};

/// Request para consultar el patrimonio consolidado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultarPatrimonioRequest {
    pub fecha_consulta: String, // Formato ISO 8601 (ej. "2026-10-23")
}

/// DTO con el resultado completo de la consulta de patrimonio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultarPatrimonioResponse {
    pub moneda_base: String,
    pub saldos_por_cuenta: Vec<SaldoCuentaDTO>,
    pub subtotales_por_moneda: Vec<SubtotalMonedaDTO>,
    /// None si falta alguna tasa (INV-02: no se puede consolidar)
    /// Some con el total redondeado si todas las tasas están disponibles
    pub total_consolidado: Option<String>,
    /// Tasas que se usaron para el cálculo (valor, fecha, fuente)
    pub tasas_usadas: Vec<TasaUsadaDTO>,
    /// Pares de monedas para los que falta una tasa
    pub tasas_faltantes: Vec<TasaFaltanteDTO>,
}

/// Saldo de una cuenta individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaldoCuentaDTO {
    pub cuenta_id: String,
    pub nombre_cuenta: String,
    pub moneda: String,
    pub saldo: String, // INV-03: String
}

/// Subtotal consolidado por moneda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtotalMonedaDTO {
    pub moneda: String,
    pub subtotal: String, // INV-03: String
}

/// Detalle de una tasa usada en el cálculo del patrimonio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasaUsadaDTO {
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: String, // INV-03: String
    pub fecha: String,
    pub fuente: String,
    pub es_inversa: bool, // true si se usó 1/valor (inversa)
}

/// Aviso de tasa faltante para un par de monedas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasaFaltanteDTO {
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub fecha_limite: String, // Fecha máxima para la tasa requerida
}
