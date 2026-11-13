use aplicacion::puertos::{ClienteSocial, ErrorOperacion};
use contrato::{
    AmistadCreadaDTO, CrearAmistadRequestDTO, CrearGrupoRequestDTO, DetalleGrupoDTO,
    GastoCreadoDTO, GrupoDTO, InvitacionGrupoCreadaDTO, InvitarMiembroGrupoRequestDTO,
    ListaAmistadesDTO, ListaGruposDTO, RegistrarGastoRequestDTO, RespuestaInvitacionDTO,
    RespuestaInvitacionGrupoDTO, UsuarioResumenDTO,
};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct AmistadIdRespuesta {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingRespuesta {
    pub estado: String,
    pub mensaje: String,
    pub version: String,
    pub timestamp: String,
}

#[derive(Debug, Error)]
pub enum SocialError {
    #[error("{mensaje}")]
    Servidor {
        status: Option<StatusCode>,
        mensaje: String,
    },
}

impl SocialError {
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::Servidor { status, .. } => *status,
        }
    }

    pub fn mensaje_ui(&self) -> &str {
        match self {
            Self::Servidor { mensaje, .. } => mensaje.as_str(),
        }
    }

    fn red() -> Self {
        Self::Servidor {
            status: None,
            mensaje: "No se pudo conectar con el servidor. Verifica tu conexión.".to_string(),
        }
    }

    fn respuesta(status: StatusCode, mensaje: impl Into<String>) -> Self {
        Self::Servidor {
            status: Some(status),
            mensaje: mensaje.into(),
        }
    }
}

impl From<SocialError> for ErrorOperacion {
    fn from(error: SocialError) -> Self {
        Self {
            status: error.status().map(|status| status.as_u16()),
            mensaje: error.mensaje_ui().to_string(),
        }
    }
}

pub struct ClienteSocialHttp {
    base_url: String,
    cliente: Client,
}

impl ClienteSocialHttp {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            cliente: Client::new(),
        }
    }

    pub async fn hacer_ping(&self) -> Result<PingRespuesta, String> {
        let respuesta = self
            .cliente
            .get(format!("{}/ping", self.base_url))
            .send()
            .await
            .map_err(|error| format!("Error de red al conectar con el servidor: {error}"))?;

        if !respuesta.status().is_success() {
            return Err(format!(
                "El servidor respondió con código de error: {}",
                respuesta.status()
            ));
        }

        respuesta
            .json::<PingRespuesta>()
            .await
            .map_err(|error| format!("Error al deserializar la respuesta del servidor: {error}"))
    }

    pub async fn buscar_usuario(
        &self,
        usuario: &str,
        token: &str,
    ) -> Result<UsuarioResumenDTO, SocialError> {
        let respuesta = self
            .cliente
            .get(format!("{}/usuarios", self.base_url))
            .query(&[("usuario", usuario)])
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn invitar_amistad(
        &self,
        datos: &CrearAmistadRequestDTO,
        token: &str,
    ) -> Result<AmistadCreadaDTO, SocialError> {
        let respuesta = self
            .cliente
            .post(format!("{}/amistades", self.base_url))
            .bearer_auth(token)
            .json(datos)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn responder_invitacion_amistad(
        &self,
        invitacion_id: &str,
        aceptar: bool,
        token: &str,
    ) -> Result<RespuestaInvitacionDTO, SocialError> {
        let respuesta = self
            .cliente
            .post(format!(
                "{}/amistades/{}/respuesta",
                self.base_url, invitacion_id
            ))
            .bearer_auth(token)
            .json(&serde_json::json!({ "aceptar": aceptar }))
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn listar_amistades(&self, token: &str) -> Result<ListaAmistadesDTO, SocialError> {
        let respuesta = self
            .cliente
            .get(format!("{}/amistades", self.base_url))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn eliminar_amistad(&self, amistad_id: &str, token: &str) -> Result<(), SocialError> {
        let respuesta = self
            .cliente
            .delete(format!("{}/amistades/{}", self.base_url, amistad_id))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        verificar_sin_contenido(respuesta).await
    }

    async fn obtener_id_amistad(
        &self,
        nombre_usuario: &str,
        token: &str,
    ) -> Result<String, SocialError> {
        let respuesta = self
            .cliente
            .get(format!("{}/amistades/{}/id", self.base_url, nombre_usuario))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        Ok(deserializar::<AmistadIdRespuesta>(respuesta).await?.id)
    }

    pub async fn listar_grupos(&self, token: &str) -> Result<ListaGruposDTO, SocialError> {
        let respuesta = self
            .cliente
            .get(format!("{}/grupos", self.base_url))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn crear_grupo(
        &self,
        datos: &CrearGrupoRequestDTO,
        token: &str,
    ) -> Result<GrupoDTO, SocialError> {
        let respuesta = self
            .cliente
            .post(format!("{}/grupos", self.base_url))
            .bearer_auth(token)
            .json(datos)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn invitar_miembro_grupo(
        &self,
        grupo_id: &str,
        datos: &InvitarMiembroGrupoRequestDTO,
        token: &str,
    ) -> Result<InvitacionGrupoCreadaDTO, SocialError> {
        let respuesta = self
            .cliente
            .post(format!("{}/grupos/{}/miembros", self.base_url, grupo_id))
            .bearer_auth(token)
            .json(datos)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn responder_invitacion_grupo(
        &self,
        grupo_id: &str,
        aceptar: bool,
        token: &str,
    ) -> Result<RespuestaInvitacionGrupoDTO, SocialError> {
        let respuesta = self
            .cliente
            .post(format!(
                "{}/grupos/{}/miembros/yo/respuesta",
                self.base_url, grupo_id
            ))
            .bearer_auth(token)
            .json(&serde_json::json!({ "aceptar": aceptar }))
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn registrar_gasto(
        &self,
        grupo_id: &str,
        datos: &RegistrarGastoRequestDTO,
        token: &str,
    ) -> Result<GastoCreadoDTO, SocialError> {
        let respuesta = self
            .cliente
            .post(format!("{}/grupos/{}/gastos", self.base_url, grupo_id))
            .bearer_auth(token)
            .json(datos)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }

    pub async fn obtener_detalle_grupo(
        &self,
        grupo_id: &str,
        token: &str,
    ) -> Result<DetalleGrupoDTO, SocialError> {
        let respuesta = self
            .cliente
            .get(format!("{}/grupos/{}", self.base_url, grupo_id))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| SocialError::red())?;
        deserializar(respuesta).await
    }
}

async fn deserializar<T: DeserializeOwned>(respuesta: Response) -> Result<T, SocialError> {
    let status = respuesta.status();
    let cuerpo = respuesta.text().await.map_err(|_| {
        SocialError::respuesta(status, "El servidor devolvió una respuesta ilegible.")
    })?;

    if !status.is_success() {
        return Err(error_http(status, &cuerpo));
    }

    serde_json::from_str(&cuerpo)
        .map_err(|_| SocialError::respuesta(status, "El servidor devolvió una respuesta inválida."))
}

async fn verificar_sin_contenido(respuesta: Response) -> Result<(), SocialError> {
    let status = respuesta.status();
    if status == StatusCode::NO_CONTENT {
        return Ok(());
    }

    let cuerpo = respuesta.text().await.unwrap_or_default();
    Err(error_http(status, &cuerpo))
}

fn error_http(status: StatusCode, cuerpo: &str) -> SocialError {
    if let Ok(datos) = serde_json::from_str::<serde_json::Value>(cuerpo) {
        if let Some(detail) = datos.get("detail").and_then(serde_json::Value::as_str) {
            return SocialError::respuesta(status, detail);
        }
        if let Some(mensaje) = datos.get("mensaje").and_then(serde_json::Value::as_str) {
            return SocialError::respuesta(status, mensaje);
        }
    }

    SocialError::respuesta(status, "El servidor no pudo completar la solicitud.")
}

impl ClienteSocial for ClienteSocialHttp {
    fn buscar_usuario(
        &self,
        usuario: String,
        token: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<UsuarioResumenDTO, ErrorOperacion>> + Send + '_,
        >,
    > {
        Box::pin(async move {
            ClienteSocialHttp::buscar_usuario(self, &usuario, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn invitar_amistad(
        &self,
        datos: CrearAmistadRequestDTO,
        token: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<AmistadCreadaDTO, ErrorOperacion>> + Send + '_>,
    > {
        Box::pin(async move {
            ClienteSocialHttp::invitar_amistad(self, &datos, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn responder_invitacion_amistad(
        &self,
        invitacion_id: String,
        aceptar: bool,
        token: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<RespuestaInvitacionDTO, ErrorOperacion>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            ClienteSocialHttp::responder_invitacion_amistad(self, &invitacion_id, aceptar, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn listar_amistades(
        &self,
        token: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<ListaAmistadesDTO, ErrorOperacion>> + Send + '_,
        >,
    > {
        Box::pin(async move {
            ClienteSocialHttp::listar_amistades(self, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn eliminar_amistad(
        &self,
        amistad_id: String,
        token: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ErrorOperacion>> + Send + '_>>
    {
        Box::pin(async move {
            ClienteSocialHttp::eliminar_amistad(self, &amistad_id, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn eliminar_amistad_por_usuario(
        &self,
        nombre_usuario: String,
        token: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ErrorOperacion>> + Send + '_>>
    {
        Box::pin(async move {
            let amistad_id = self
                .obtener_id_amistad(&nombre_usuario, &token)
                .await
                .map_err(ErrorOperacion::from)?;
            ClienteSocialHttp::eliminar_amistad(self, &amistad_id, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn crear_grupo(
        &self,
        datos: CrearGrupoRequestDTO,
        token: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GrupoDTO, ErrorOperacion>> + Send + '_>,
    > {
        Box::pin(async move {
            ClienteSocialHttp::crear_grupo(self, &datos, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn listar_grupos(
        &self,
        token: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ListaGruposDTO, ErrorOperacion>> + Send + '_>,
    > {
        Box::pin(async move {
            ClienteSocialHttp::listar_grupos(self, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn invitar_miembro_grupo(
        &self,
        grupo_id: String,
        datos: InvitarMiembroGrupoRequestDTO,
        token: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<InvitacionGrupoCreadaDTO, ErrorOperacion>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            ClienteSocialHttp::invitar_miembro_grupo(self, &grupo_id, &datos, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn responder_invitacion_grupo(
        &self,
        grupo_id: String,
        aceptar: bool,
        token: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<RespuestaInvitacionGrupoDTO, ErrorOperacion>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            ClienteSocialHttp::responder_invitacion_grupo(self, &grupo_id, aceptar, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn registrar_gasto(
        &self,
        grupo_id: String,
        datos: RegistrarGastoRequestDTO,
        token: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GastoCreadoDTO, ErrorOperacion>> + Send + '_>,
    > {
        Box::pin(async move {
            ClienteSocialHttp::registrar_gasto(self, &grupo_id, &datos, &token)
                .await
                .map_err(Into::into)
        })
    }

    fn obtener_detalle_grupo(
        &self,
        grupo_id: String,
        token: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<DetalleGrupoDTO, ErrorOperacion>> + Send + '_>,
    > {
        Box::pin(async move {
            ClienteSocialHttp::obtener_detalle_grupo(self, &grupo_id, &token)
                .await
                .map_err(Into::into)
        })
    }
}
