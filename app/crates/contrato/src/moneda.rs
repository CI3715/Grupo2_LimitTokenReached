use serde::{Deserialize, Serialize};

// estas son mis monedas protegidas
pub const CODIGOS_MONEDAS_CATALOGO: [&str; 4] = ["VES","USD","EUR","USDT"] ;
/// DTO para mostrar una moneda (del catálogo o propia)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonedaDTO {
    pub codigo: String,        // 3-5 letras mayúsculas
    pub nombre: String,        // Nombre completo
    pub simbolo: String,       // Símbolo
    pub decimales: u32,        // cambio: u8 → u32 (compatible con rust_decimal::scale())
    pub es_del_catalogo: bool, // true si es VES/USD/EUR/USDT (no modificable)
}

/// Request para crear una moneda propia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliminarMonedaRequest {
    pub codigo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrearMonedaRequest {
    pub codigo: String, // 3-5 letras mayúsculas, único
    pub nombre: String,
    pub simbolo: String,
    pub decimales: u32, // cambio: u8 → u32
}

/// Request para cambiar la moneda base del usuario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CambiarMonedaBaseRequest {
    pub codigo: String, // Código de la moneda a establecer como base
}
