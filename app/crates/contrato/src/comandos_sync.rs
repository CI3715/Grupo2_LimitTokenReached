use crate::respaldo_sincronizacion::{EventoRechazadoDTO, ReporteSincronizacionDTO};
use serde::{Deserialize, Serialize};

/// request para exportar el libro a un archivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportarLibroRequest {
    pub ruta_destino: String,
    pub frase_de_paso: Option<String>, // none si es sin cifrar
    pub cifrar: bool,
}

/// response tras exportar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportarLibroResponse {
    pub exitoso: bool,
    pub ruta_archivo: String,
    pub cantidad_eventos: u64,
}

/// request para importar un archivo al libro local
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportarLibroRequest {
    pub ruta_archivo: String,
    pub frase_de_paso: String,
}

/// response tras importar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportarLibroResponse {
    pub exitoso: bool,
    pub reporte: ReporteSincronizacionDTO,
}

/// configuración del destino de sincronización
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDestinoSyncDTO {
    pub tipo: String, // "git" o "carpeta"
    pub ruta_o_url: String,
    pub credenciales: Option<String>, // token o usuario:pass
}

/// resultado de una operación de sincronización
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoSyncDTO {
    pub exitoso: bool,
    pub eventos_enviados: u64,
    pub eventos_recibidos: u64,
    pub eventos_rechazados: Vec<EventoRechazadoDTO>,
    pub fecha_ultima_sync: String,
}
