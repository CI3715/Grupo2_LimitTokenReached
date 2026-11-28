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
    pub tipo: TipoMovimientoDTO,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrarMovimientoRequest {
    pub cuenta_id: String,
    pub tipo: TipoMovimientoDTO,
    pub monto: i64,                // INV-03: i64 (multiplicado por 10^8)
    pub fecha: String,
    pub categoria_id: Option<String>,
    pub nota: Option<String>,
    #[serde(default)]
    pub confirmar_saldo_negativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorregirMovimientoRequest {
    pub movimiento_id: String,
    pub monto_correcto: i64,       // INV-03: i64
    #[serde(default)]
    pub confirmar_saldo_negativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovimientoDTO {
    pub id: String,
    pub cuenta_id: String,
    pub tipo: TipoMovimientoDTO,
    pub monto: i64,                // INV-03: i64
    pub monto_resultante: i64,     // INV-03: i64
    pub fecha: String,
    pub categoria_id: Option<String>,
    pub nota: Option<String>,
    pub es_correccion: bool,
    pub movimiento_original_id: Option<String>,
    pub gasto_grupo: bool,
    pub grupo_id: Option<String>,
    pub grupo_gasto_id: Option<String>,
}