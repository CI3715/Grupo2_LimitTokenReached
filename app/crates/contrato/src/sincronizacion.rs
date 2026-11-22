use serde::{Deserialize, Serialize};

/// evento de cambio recibido del servidor para sincronizar el libro personal
/// cumple con idempotencia y sincronización sin duplicados
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventoCambioDTO {
    pub id_evento: String,             // identificador único e inmutable (INV-13)
    pub tipo: String,                  // ej. "GastoDeGrupoRegistrado"
    pub grupo_id: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,                 // formato ISO 8601
    pub moneda: String,
    pub monto_parte: String,           // INV-03: string para precisión decimal exacta
    pub incluye_redondeo: bool,        // marca de céntimos de residuo 
    pub posicion_cursor: u64,          // cursor creciente para paginación
}

/// respuesta del endpoint GET /cambios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespuestaCambiosDTO {
    pub eventos: Vec<EventoCambioDTO>, // max 100 eventos por lote
    pub nuevo_cursor: u64,             // cursor para la próxima petición
    pub hay_mas: bool,                 // es true si quedan más eventos por sincronizar
}

/// query params para GET /cambios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObtenerCambiosRequest {
    pub desde: Option<u64>,            // cursor a partir del cual obtener eventos
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ParteGrupoPendienteDTO {     // representa una parte que llegó sin cuenta elegida.
    pub id_evento: String,
    pub grupo_id: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,
    pub moneda: String,
    pub monto_parte: String,
    pub incluye_redondeo: bool,
    pub posicion_cursor: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CuentaGrupoSeleccionDTO {    // epresenta la selección local del usuario.
    pub grupo_id: String,
    pub cuenta_id: String,
    pub moneda: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EstadoSincronizacionDTO {    // permite mostrar cursor, última sincronización y vencimiento de sesión.
    pub cursor: u64,
    pub ultima_sincronizacion: Option<String>,
    pub sesion_vencida: bool,
}