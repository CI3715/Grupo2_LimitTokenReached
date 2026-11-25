use serde::{Deserialize, Serialize};

/// encabezado del archivo de respaldo exportado
/// cumple con texto estructurado con versión, fecha y cantidad
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncabezadoExportacionDTO {
    pub version_formato: String,
    pub fecha_exportacion: String,
    pub cantidad_eventos: u64,
}

/// evento que no pudo fusionarse durante la importación o sincronización
/// esto se rechaza y se informa el motivo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventoRechazadoDTO {
    pub id_evento: String,
    pub motivo: String,
}

/// reporte final tras importar o sincronizar
/// informa nuevos, existentes, rechazados y fecha
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReporteSincronizacionDTO {
    pub eventos_nuevos: u64,
    pub eventos_ya_existentes: u64,
    pub eventos_rechazados: Vec<EventoRechazadoDTO>,
    pub fecha_ultima_sincronizacion: String,
}