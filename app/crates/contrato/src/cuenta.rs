use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CuentaDTO {
    pub id: String,
    pub nombre: String,
    pub tipo: TipoCuentaDTO,
    pub moneda_codigo: String,
    pub saldo: i64,                // INV-03: i64 (multiplicado por 10^8)
    pub creada_en: String,
    pub activa: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TipoCuentaDTO {
    Efectivo,
    Banco,
    BilleteraDigital,
}