use serde::{Deserialize, Serialize};

/// catálogo de errores de negocio para Sincronización y Respaldo
/// estos errores viajan al frontend para mostrar mensajes claros sin trazas técnicas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSincronizacion {
    /// frase de paso incorrecta o archivo alterado
    FraseIncorrectaOArchivoDanado,

    /// el archivo tiene una versión de formato que esta app no soporta
    VersionFormatoNoSoportada {
        version_archivo: String,
        version_soportada: String,
    },

    /// no se pudo acceder al destino (Git remoto caído o carpeta borrada)
    DestinoInaccesibleOSinConexion,

    /// el token o usuario de Git fue rechazado por el remoto
    CredencialesGitInvalidas,

    /// conflicto: mismo ID de evento pero contenido distinto
    EventoConflictoContenido { id_evento: String },
}

impl std::fmt::Display for ErrorSincronizacion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSincronizacion::FraseIncorrectaOArchivoDanado => {
                write!(f, "Frase de paso incorrecta o archivo dañado.")
            }
            ErrorSincronizacion::VersionFormatoNoSoportada {
                version_archivo,
                version_soportada,
            } => write!(
                f,
                "Versión de formato no soportada. El archivo es v{} y la app soporta hasta v{}.",
                version_archivo, version_soportada
            ),
            ErrorSincronizacion::DestinoInaccesibleOSinConexion => write!(
                f,
                "No se pudo llegar al repositorio o a la carpeta; se reintentará."
            ),
            ErrorSincronizacion::CredencialesGitInvalidas => write!(
                f,
                "Las credenciales fueron rechazadas; por favor actualízalas en configuración."
            ),
            ErrorSincronizacion::EventoConflictoContenido { id_evento } => write!(
                f,
                "Evento con identificador repetido y contenido distinto (ID: {}).",
                id_evento
            ),
        }
    }
}
