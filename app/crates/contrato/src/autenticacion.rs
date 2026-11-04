//! Contrato E5 para la frontera UI/Tauri/servidor.
//! Las reglas de registro y las credenciales se validan en el servidor.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RegistrarUsuarioRequest {
    pub nombre_usuario: String,
    pub nombre_visible: String,
    pub correo: String,
    pub contrasena: String,
}

impl fmt::Debug for RegistrarUsuarioRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrarUsuarioRequest")
            .field("nombre_usuario", &self.nombre_usuario)
            .field("nombre_visible", &self.nombre_visible)
            .field("correo", &self.correo)
            .field("contrasena", &"[REDACTADA]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IniciarSesionRequest {
    pub correo: String,
    pub contrasena: String,
}

impl fmt::Debug for IniciarSesionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IniciarSesionRequest")
            .field("correo", &self.correo)
            .field("contrasena", &"[REDACTADA]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UsuarioDTO {
    pub id: String,
    pub nombre_usuario: String,
    pub nombre_visible: String,
    pub correo: String,
    pub fecha_alta: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SesionDTO {
    pub token: String,
    pub vence_en: String,
}

impl fmt::Debug for SesionDTO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SesionDTO")
            .field("token", &"[REDACTADO]")
            .field("vence_en", &self.vence_en)
            .finish()
    }
}
