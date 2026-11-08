use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Serialize)]
pub struct RegistroUsuarioHttp {
    pub nombre_usuario: String,
    pub nombre_visible: String,
    pub correo: String,
    pub contrasena: String,
}

#[derive(Serialize)]
pub struct InicioSesionHttp {
    pub correo: String,
    pub contrasena: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsuarioHttp {
    pub id: String,
    pub nombre_usuario: String,
    pub nombre_visible: String,
    pub correo: String,
    pub fecha_alta: String,
}

#[derive(Deserialize)]
pub struct SesionHttp {
    pub token: String,
    pub vence_en: String,
}

#[derive(Deserialize, Serialize)]
struct ErrorHttp {
    campo: String,
    mensaje: String,
}

pub struct ErrorApi {
    pub status: Option<StatusCode>,
    campo: Option<String>,
    mensaje: String,
}

impl ErrorApi {
    pub fn status(&self) -> Option<StatusCode> {
        self.status
    }

    pub fn mensaje_ui(&self) -> String {
        if let Some(campo) = &self.campo {
            return serde_json::to_string(&ErrorHttp {
                campo: campo.clone(),
                mensaje: self.mensaje.clone(),
            })
            .unwrap_or_else(|_| "No se pudo completar la solicitud.".to_string());
        }
        self.mensaje.clone()
    }
}

pub struct ClienteAutenticacionHttp {
    base_url: String,
    cliente: Client,
}

impl ClienteAutenticacionHttp {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            cliente: Client::new(),
        }
    }

    pub async fn registrar_usuario(
        &self,
        datos: &RegistroUsuarioHttp,
    ) -> Result<UsuarioHttp, ErrorApi> {
        let respuesta = self
            .cliente
            .post(format!("{}/usuarios", self.base_url))
            .json(datos)
            .send()
            .await
            .map_err(|_| Self::error_red())?;
        Self::deserializar(respuesta).await
    }

    pub async fn iniciar_sesion(
        &self,
        datos: &InicioSesionHttp,
    ) -> Result<SesionHttp, ErrorApi> {
        let respuesta = self
            .cliente
            .post(format!("{}/sesiones", self.base_url))
            .json(datos)
            .send()
            .await
            .map_err(|_| Self::error_red())?;
        Self::deserializar(respuesta).await
    }

    pub async fn obtener_perfil(&self, token: &str) -> Result<UsuarioHttp, ErrorApi> {
        let respuesta = self
            .cliente
            .get(format!("{}/usuarios/yo", self.base_url))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| Self::error_red())?;
        Self::deserializar(respuesta).await
    }

    async fn deserializar<T: DeserializeOwned>(respuesta: Response) -> Result<T, ErrorApi> {
        let status = respuesta.status();
        let cuerpo = respuesta.text().await.map_err(|_| ErrorApi {
            status: Some(status),
            campo: None,
            mensaje: "El servidor devolvió una respuesta ilegible.".to_string(),
        })?;

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ErrorHttp>(&cuerpo) {
                return Err(ErrorApi {
                    status: Some(status),
                    campo: Some(error.campo),
                    mensaje: error.mensaje,
                });
            }
            return Err(ErrorApi {
                status: Some(status),
                campo: None,
                mensaje: "El servidor no pudo completar la solicitud.".to_string(),
            });
        }

        serde_json::from_str(&cuerpo).map_err(|_| ErrorApi {
            status: Some(status),
            campo: None,
            mensaje: "El servidor devolvió una respuesta inválida.".to_string(),
        })
    }

    fn error_red() -> ErrorApi {
        ErrorApi {
            status: None,
            campo: None,
            mensaje: "No se pudo conectar con el servidor. Verifica tu conexión.".to_string(),
        }
    }
}
