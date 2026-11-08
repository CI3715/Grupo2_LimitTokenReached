use crate::{IniciarSesionRequest, RegistrarUsuarioRequest, SesionDTO, UsuarioDTO};
use serde_json::{Value, json};

fn contratos() -> Value {
    serde_json::from_str(include_str!("../../../../qa/fixtures/autenticacion.json")).unwrap()
}

#[test]
fn registro_e_inicio_serializan_el_mismo_contrato_que_espera_python() {
    let datos = contratos();
    let registro: RegistrarUsuarioRequest =
        serde_json::from_value(datos["registro"].clone()).unwrap();
    let inicio: IniciarSesionRequest =
        serde_json::from_value(datos["inicio_sesion"].clone()).unwrap();
    // Para enviarla al servidor, la contraseña viaja en el cuerpo autorizado.
    // No se debe registrar el cuerpo JSON de estas solicitudes.
    assert_eq!(serde_json::to_value(&registro).unwrap(), datos["registro"]);
    assert_eq!(
        serde_json::to_value(&inicio).unwrap(),
        datos["inicio_sesion"]
    );
    assert!(!format!("{registro:?}").contains(&registro.contrasena));
    assert!(!format!("{inicio:?}").contains(&inicio.contrasena));
}

#[test]
fn perfil_y_sesion_leen_la_respuesta_del_servidor() {
    let datos = contratos();
    let usuario: UsuarioDTO = serde_json::from_value(datos["usuario"].clone()).unwrap();
    let sesion: SesionDTO = serde_json::from_value(datos["sesion"].clone()).unwrap();
    assert_eq!(serde_json::to_value(usuario).unwrap(), datos["usuario"]);
    assert_eq!(serde_json::to_value(&sesion).unwrap(), datos["sesion"]);
    assert!(!format!("{sesion:?}").contains(&sesion.token));
}

#[test]
fn requests_de_auth_rechazan_datos_financieros_adicionales() {
    for campo in ["cuentas", "saldos", "movimientos", "tasas"] {
        let mut registro = contratos()["registro"].clone();
        registro[campo] = json!([]);
        assert!(serde_json::from_value::<RegistrarUsuarioRequest>(registro).is_err());
        let mut inicio = contratos()["inicio_sesion"].clone();
        inicio[campo] = json!([]);
        assert!(serde_json::from_value::<IniciarSesionRequest>(inicio).is_err());
    }
}

#[test]
fn perfil_no_admite_id_ni_campos_secretos() {
    for campo in ["id", "contrasena", "password", "hash", "hash_contrasena"] {
        let mut usuario = contratos()["usuario"].clone();
        usuario[campo] = json!("secreto");
        assert!(serde_json::from_value::<UsuarioDTO>(usuario).is_err());
    }
}

#[test]
fn campo_y_mensaje_de_error_coinciden_con_el_contrato_de_ui() {
    let datos = contratos();
    let error: crate::ErrorValidacionDTO =
        serde_json::from_value(datos["credenciales_invalidas"].clone()).unwrap();
    assert_eq!(error.campo, "credenciales");
    assert_eq!(error.mensaje, "Correo o contraseña incorrectos.");
}

#[test]
fn un_request_sin_datos_obligatorios_no_se_deserializa() {
    for campo in ["nombre_usuario", "nombre_visible", "correo", "contrasena"] {
        let mut registro = contratos()["registro"].clone();
        registro.as_object_mut().unwrap().remove(campo);
        assert!(serde_json::from_value::<RegistrarUsuarioRequest>(registro).is_err());
    }
    for campo in ["correo", "contrasena"] {
        let mut inicio = contratos()["inicio_sesion"].clone();
        inicio.as_object_mut().unwrap().remove(campo);
        assert!(serde_json::from_value::<IniciarSesionRequest>(inicio).is_err());
    }
}
