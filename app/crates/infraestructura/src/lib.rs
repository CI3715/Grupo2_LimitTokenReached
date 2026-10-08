use serde::{Deserialize, Serialize};
use aplicacion::puertos::Reloj; // Importa el Trait definido en aplicacion
use chrono::Utc;


// Adaptador Real del Reloj del Sistema
pub struct RelojSistema;

impl Reloj for RelojSistema {
    fn ahora(&self) -> String {
        Utc::now().to_rfc3339() // Devuelve la fecha y hora real UTC en formato ISO 8601
    }
}

// Estructura que mapea la respuesta JSON que enviará el servidor FastAPI en el endpoint GET /ping
#[derive(Debug, Serialize, Deserialize)]
pub struct PingRespuesta {
    pub estado: String,
    pub mensaje: String,
    pub version: String,
    pub timestamp: String,
}

/// Adaptador HTTP que realiza las peticiones hacia el Servidor (FastAPI)
pub struct ClienteSocialHttp {
    base_url: String,
}

impl ClienteSocialHttp {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// Realiza la petición GET /ping al servidor y deserializa la respuesta
    pub async fn hacer_ping(&self) -> Result<PingRespuesta, String> {
        let url = format!("{}/ping", self.base_url);

        let respuesta = reqwest::get(&url)
            .await
            .map_err(|e| format!("Error de red al conectar con el servidor: {e}"))?;

        if !respuesta.status().is_success() {
            return Err(format!(
                "El servidor respondió con código de error: {}",
                respuesta.status()
            ));
        }

        respuesta
            .json::<PingRespuesta>()
            .await
            .map_err(|e| format!("Error al deserializar la respuesta del servidor: {e}"))
    }
}