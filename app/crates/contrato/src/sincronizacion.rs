use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventoCambioDTO {
    pub id_evento: String,
    pub tipo: String,
    pub grupo_id: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,
    pub moneda: String,
    pub monto_parte: i64,              // INV-03: i64 (multiplicado por 10^8)
    pub incluye_redondeo: bool,
    pub posicion_cursor: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespuestaCambiosDTO {
    pub eventos: Vec<EventoCambioDTO>,
    pub nuevo_cursor: u64,
    pub hay_mas: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObtenerCambiosRequest {
    pub desde: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParteGrupoPendienteDTO {
    pub id_evento: String,
    pub grupo_id: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,
    pub moneda: String,
    pub monto_parte: i64,              // INV-03: i64 (multiplicado por 10^8)
    pub incluye_redondeo: bool,
    pub posicion_cursor: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CuentaGrupoSeleccionDTO {
    pub grupo_id: String,
    pub cuenta_id: String,
    pub moneda: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EstadoSincronizacionDTO {
    pub cursor: u64,
    pub ultima_sincronizacion: Option<String>,
    pub sesion_vencida: bool,
}