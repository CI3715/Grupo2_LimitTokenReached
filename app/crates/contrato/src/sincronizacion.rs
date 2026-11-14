use serde::{Deserialize, Serialize};

/// evento de cambio recibido del servidor para sincronizar el libro personal
/// cumple con idempotencia y sincronización sin duplicados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventoCambioDTO {
    pub id_evento: String,             // id único e inmutable (INV-13)
    pub tipo: String,                  // ej. "GastoDeGrupoRegistrado"
    pub grupo_id: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,                 // Formato ISO 8601
    pub moneda: String,
    pub monto_parte: String,           // INV-03: string para precisión decimal exacta
    pub incluye_redondeo: bool,        // marca de céntimos de residuo 
    pub posicion_cursor: u64,          // cursor creciente para paginación
}

/// respuesta del endpoint GET /cambios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaCambiosDTO {
    pub eventos: Vec<EventoCambioDTO>, // máx 100 eventos por lote
    pub nuevo_cursor: u64,             // cursor para la próxima petición
    pub hay_mas: bool,                 // es true si quedan más eventos por sincronizar
}

/// query params para GET /cambios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObtenerCambiosRequest {
    pub desde: Option<u64>,            // cursor a partir del cual obtener eventos
}