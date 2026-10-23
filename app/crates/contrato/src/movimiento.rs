use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TipoMovimientoDTO {
    Ingreso,
    Gasto,
    Ajuste,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriaDTO {
    pub id: String,
    pub nombre: String,
    pub tipo: TipoMovimientoDTO, // "Gasto" o "Ingreso"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrarMovimientoRequest {
    pub cuenta_id: String,
    pub tipo: TipoMovimientoDTO,
    pub monto: String,                // INV-03: String para evitar f32/f64
    pub fecha: String,                // Formato ISO 8601 (ej. "2026-10-19")
    pub categoria_id: Option<String>, //se modifico para que fuera consistente con el struct DatosMovimientos en /dominio/src/movimiento
    pub nota: Option<String>,
    #[serde(default)]
    pub confirmar_saldo_negativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorregirMovimientoRequest {
    pub movimiento_id: String,
    pub monto_correcto: String, // INV-03: String
    #[serde(default)]
    pub confirmar_saldo_negativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovimientoDTO {
    pub id: String,
    pub cuenta_id: String,
    pub tipo: TipoMovimientoDTO,
    pub monto: String,
    pub fecha: String,
    pub categoria_id: Option<String>,
    pub nota: Option<String>,
    pub es_correccion: bool,
    pub movimiento_original_id: Option<String>, // None si es original, Some(id) si es ajuste
}
