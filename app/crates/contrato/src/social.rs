use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UsuarioResumenDTO {
    pub nombre_usuario: String,
    pub nombre_visible: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrearAmistadRequestDTO {
    pub usuario: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResponderInvitacionRequestDTO {
    pub aceptar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AmistadCreadaDTO {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RespuestaInvitacionDTO {
    pub estado: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvitacionAmistadDTO {
    pub id: String,
    pub nombre_usuario: String,
    pub nombre_visible: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListaAmistadesDTO {
    pub amigos: Vec<UsuarioResumenDTO>,
    pub enviadas: Vec<InvitacionAmistadDTO>,
    pub recibidas: Vec<InvitacionAmistadDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrearGrupoRequestDTO {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub moneda: String,
    pub actividad: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GrupoDTO {
    pub id: String,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub moneda: String,
    pub actividad: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListaGruposDTO {
    pub miembros: Vec<GrupoDTO>,
    pub invitaciones: Vec<GrupoDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvitarMiembroGrupoRequestDTO {
    pub usuario: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvitacionGrupoCreadaDTO {
    pub id: String,
    pub invitacion_id: String,
    pub usuario: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RespuestaInvitacionGrupoDTO {
    pub id: String,
    pub estado: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RegistrarGastoRequestDTO {
    pub concepto: String,
    pub monto: String,
    pub fecha: String,
    pub pagador: String,
    pub regla: String,
    pub participantes: Vec<String>,
    pub valores: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParteGastoDTO {
    pub usuario: String,
    pub monto: String,
    pub residuo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GastoDTO {
    pub id: String,
    pub concepto: String,
    pub monto: String,
    pub fecha: String,
    pub pagador: String,
    pub regla: String,
    pub partes: Vec<ParteGastoDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GastoCreadoDTO {
    pub id: String,
    pub partes: Vec<ParteGastoDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MiembroGrupoDTO {
    pub nombre_usuario: String,
    pub nombre_visible: String,
    pub rol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DetalleGrupoDTO {
    pub id: String,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub moneda: String,
    pub actividad: Option<String>,
    pub miembros: Vec<MiembroGrupoDTO>,
    pub gastos: Vec<GastoDTO>,
    pub balances: HashMap<String, String>,
    pub partes: Vec<ParteGastoDTO>,
}
