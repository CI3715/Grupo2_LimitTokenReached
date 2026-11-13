use crate::puertos::ClienteSocial;
use contrato::{
    AmistadCreadaDTO, CrearAmistadRequestDTO, CrearGrupoRequestDTO, DetalleGrupoDTO,
    GastoCreadoDTO, GrupoDTO, InvitacionGrupoCreadaDTO, InvitarMiembroGrupoRequestDTO,
    ListaAmistadesDTO, RegistrarGastoRequestDTO, RespuestaInvitacionDTO,
    RespuestaInvitacionGrupoDTO, UsuarioResumenDTO,
};
use rust_decimal::Decimal;
use std::str::FromStr;

pub async fn buscar_usuario(
    cliente: &dyn ClienteSocial,
    usuario: String,
    token: String,
) -> Result<UsuarioResumenDTO, crate::puertos::ErrorOperacion> {
    cliente.buscar_usuario(usuario, token).await
}

pub async fn invitar_amistad(
    cliente: &dyn ClienteSocial,
    datos: CrearAmistadRequestDTO,
    token: String,
) -> Result<AmistadCreadaDTO, crate::puertos::ErrorOperacion> {
    cliente.invitar_amistad(datos, token).await
}

pub async fn responder_invitacion_amistad(
    cliente: &dyn ClienteSocial,
    invitacion_id: String,
    aceptar: bool,
    token: String,
) -> Result<RespuestaInvitacionDTO, crate::puertos::ErrorOperacion> {
    cliente
        .responder_invitacion_amistad(invitacion_id, aceptar, token)
        .await
}

pub async fn listar_amistades(
    cliente: &dyn ClienteSocial,
    token: String,
) -> Result<ListaAmistadesDTO, crate::puertos::ErrorOperacion> {
    cliente.listar_amistades(token).await
}

pub async fn eliminar_amistad(
    cliente: &dyn ClienteSocial,
    amistad_id: String,
    token: String,
) -> Result<(), crate::puertos::ErrorOperacion> {
    cliente.eliminar_amistad(amistad_id, token).await
}

pub async fn eliminar_amistad_por_usuario(
    cliente: &dyn ClienteSocial,
    nombre_usuario: String,
    token: String,
) -> Result<(), crate::puertos::ErrorOperacion> {
    cliente
        .eliminar_amistad_por_usuario(nombre_usuario, token)
        .await
}

pub async fn crear_grupo(
    cliente: &dyn ClienteSocial,
    datos: CrearGrupoRequestDTO,
    token: String,
) -> Result<GrupoDTO, crate::puertos::ErrorOperacion> {
    cliente.crear_grupo(datos, token).await
}

pub async fn invitar_miembro_grupo(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    datos: InvitarMiembroGrupoRequestDTO,
    token: String,
) -> Result<InvitacionGrupoCreadaDTO, crate::puertos::ErrorOperacion> {
    cliente.invitar_miembro_grupo(grupo_id, datos, token).await
}

pub async fn responder_invitacion_grupo(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    aceptar: bool,
    token: String,
) -> Result<RespuestaInvitacionGrupoDTO, crate::puertos::ErrorOperacion> {
    cliente
        .responder_invitacion_grupo(grupo_id, aceptar, token)
        .await
}

pub async fn registrar_gasto(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    datos: RegistrarGastoRequestDTO,
    token: String,
) -> Result<GastoCreadoDTO, crate::puertos::ErrorOperacion> {
    cliente.registrar_gasto(grupo_id, datos, token).await
}

pub async fn obtener_detalle_grupo(
    cliente: &dyn ClienteSocial,
    grupo_id: String,
    token: String,
) -> Result<DetalleGrupoDTO, crate::puertos::ErrorOperacion> {
    let detalle = cliente.obtener_detalle_grupo(grupo_id, token).await?;
    validar_balances(&detalle.balances)?;
    Ok(detalle)
}

fn validar_balances(
    balances: &std::collections::HashMap<String, String>,
) -> Result<(), crate::puertos::ErrorOperacion> {
    let mut suma = Decimal::ZERO;

    for (usuario, valor) in balances {
        let balance = Decimal::from_str(valor).map_err(|_| crate::puertos::ErrorOperacion {
            status: None,
            mensaje: format!("El balance de {usuario} no es un decimal válido."),
        })?;
        suma += balance;
    }

    if suma != Decimal::ZERO {
        return Err(crate::puertos::ErrorOperacion {
            status: None,
            mensaje: format!(
                "La suma de los balances del grupo debe ser exactamente cero; se obtuvo {suma}."
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validar_balances;
    use std::collections::HashMap;

    #[test]
    fn acepta_balances_cuya_suma_es_exactamente_cero() {
        let balances = HashMap::from([
            ("ana".to_string(), "36.66".to_string()),
            ("beto".to_string(), "11.67".to_string()),
            ("carla".to_string(), "-48.33".to_string()),
        ]);

        assert!(validar_balances(&balances).is_ok());
    }

    #[test]
    fn rechaza_balances_cuya_suma_no_es_cero() {
        let balances = HashMap::from([
            ("ana".to_string(), "36.66".to_string()),
            ("beto".to_string(), "11.68".to_string()),
            ("carla".to_string(), "-48.33".to_string()),
        ]);

        let error = validar_balances(&balances).unwrap_err();

        assert!(error.mensaje.contains("exactamente cero"));
    }

    #[test]
    fn rechaza_un_balance_que_no_es_decimal() {
        let balances = HashMap::from([("ana".to_string(), "invalido".to_string())]);

        let error = validar_balances(&balances).unwrap_err();

        assert!(error.mensaje.contains("ana"));
        assert!(error.mensaje.contains("decimal válido"));
    }
}
