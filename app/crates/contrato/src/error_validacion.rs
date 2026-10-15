use serde::{Deserialize, Serialize};

/// Error de validación devuelto al frontend
/// Contiene el campo que falló y un mensaje legible para el usuario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorValidacionDTO {
    pub campo: String,        // "nombre", "moneda_codigo", "saldo_inicial"
    pub mensaje: String,      // "El nombre no puede estar vacío"
}