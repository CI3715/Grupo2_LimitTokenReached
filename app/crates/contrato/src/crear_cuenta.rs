use serde::{Deserialize, Serialize};

use super::cuenta::TipoCuentaDTO;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrearCuentaRequest {
    pub nombre: String,
    pub tipo: TipoCuentaDTO,
    pub moneda_codigo: String,
    pub saldo_inicial: i64,        // INV-03: i64 (multiplicado por 10^8)
}