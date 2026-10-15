use serde::{Deserialize, Serialize};

use super::cuenta::TipoCuentaDTO;

/// Datos que el frontend envía para crear una cuenta
/// Las validaciones reales las hace el dominio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrearCuentaRequest {
    pub nombre: String,
    pub tipo: TipoCuentaDTO,
    pub moneda_codigo: String,
    pub saldo_inicial: String, // string para preservar decimales exactos
}