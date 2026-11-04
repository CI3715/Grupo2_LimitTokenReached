import json
from typing import Any

import pytest
from pydantic import ValidationError

from cuentas_claras.contrato.autenticacion import (
    ErrorAutenticacionDTO,
    IniciarSesionRequest,
    RegistrarUsuarioRequest,
    SesionDTO,
    UsuarioDTO,
    error_credenciales_invalidas,
    error_datos_invalidos,
)


def test_los_dtos_python_aceptan_el_contrato_compartido(
    contratos_auth: dict[str, Any],
) -> None:
    RegistrarUsuarioRequest.model_validate(contratos_auth["registro"])
    IniciarSesionRequest.model_validate(contratos_auth["inicio_sesion"])
    assert (
        UsuarioDTO.model_validate(contratos_auth["usuario"]).id == "usuario-de-prueba"
    )
    assert (
        SesionDTO.model_validate(contratos_auth["sesion"]).vence_en.tzinfo is not None
    )
    assert (
        error_credenciales_invalidas().model_dump()
        == contratos_auth["credenciales_invalidas"]
    )


@pytest.mark.parametrize("dto", [RegistrarUsuarioRequest, IniciarSesionRequest])
def test_contrasena_no_aparece_en_repr_dump_ni_log_de_request(
    contratos_auth: dict[str, Any],
    dto: type[RegistrarUsuarioRequest] | type[IniciarSesionRequest],
    caplog: pytest.LogCaptureFixture,
) -> None:
    import logging

    clave = "ContrasenaSoloParaPruebas123"
    fixture = "registro" if dto is RegistrarUsuarioRequest else "inicio_sesion"
    request = dto.model_validate(dict(contratos_auth[fixture], contrasena=clave))
    logging.getLogger(__name__).warning("Request validado: %r", request)
    assert clave not in caplog.text
    assert clave not in repr(request)
    assert clave not in request.model_dump_json()
    assert "contrasena" not in request.model_dump()
    assert request.contrasena.get_secret_value() == clave


@pytest.mark.parametrize("campo", ["nombre_usuario", "correo", "contrasena"])
def test_errores_serializables_no_filtran_la_clave_ni_el_body(
    contratos_auth: dict[str, Any], campo: str
) -> None:
    clave = "secreto9" if campo == "contrasena" else "secreto-de-prueba-no-publico"
    datos = dict(contratos_auth["registro"], contrasena=clave)
    if campo != "contrasena":
        datos[campo] = "X"
    with pytest.raises(ValidationError) as resultado:
        RegistrarUsuarioRequest.model_validate(datos)
    error = error_datos_invalidos(resultado.value.errors())
    assert clave not in str(resultado.value)
    assert clave not in error.model_dump_json()
    assert set(error.model_dump()) == {"campo", "mensaje"}


def test_sanitizador_no_reenvia_mensajes_o_contextos_del_validador() -> None:
    error = error_datos_invalidos(
        [
            {
                "type": "desconocido",
                "msg": "clave-no-publica",
                "input": "clave-no-publica",
            }
        ]
    )
    assert error.campo == "general"
    assert "clave-no-publica" not in error.model_dump_json()
    assert error_datos_invalidos([]).mensaje


@pytest.mark.parametrize("campo", ["cuentas", "saldos", "movimientos", "tasas"])
@pytest.mark.parametrize("dto", [RegistrarUsuarioRequest, IniciarSesionRequest])
def test_contrato_de_auth_rechaza_datos_financieros(
    contratos_auth: dict[str, Any],
    campo: str,
    dto: type[RegistrarUsuarioRequest] | type[IniciarSesionRequest],
) -> None:
    fixture = "registro" if dto is RegistrarUsuarioRequest else "inicio_sesion"
    with pytest.raises(ValidationError) as resultado:
        dto.model_validate(dict(contratos_auth[fixture], **{campo: []}))
    assert error_datos_invalidos(resultado.value.errors()).campo == "general"


@pytest.mark.parametrize("campo", ["contrasena", "password", "hash_contrasena", "hash"])
def test_usuario_publico_rechaza_campos_secretos(
    contratos_auth: dict[str, Any], campo: str
) -> None:
    with pytest.raises(ValidationError):
        UsuarioDTO.model_validate(dict(contratos_auth["usuario"], **{campo: "secreto"}))
    usuario = UsuarioDTO.model_validate(contratos_auth["usuario"])
    assert set(usuario.model_dump()) == set(contratos_auth["usuario"])


@pytest.mark.parametrize("fecha", ["2026-11-04T13:00:00", "fecha-invalida", ""])
def test_vencimiento_del_contrato_exige_fecha_con_zona(
    contratos_auth: dict[str, Any], fecha: str
) -> None:
    with pytest.raises(ValidationError):
        SesionDTO.model_validate(dict(contratos_auth["sesion"], vence_en=fecha))


def test_respuesta_de_sesion_entrega_token_pero_no_lo_muestra_en_repr(
    contratos_auth: dict[str, Any],
) -> None:
    sesion = SesionDTO.model_validate(contratos_auth["sesion"])
    assert sesion.model_dump()["token"] == contratos_auth["sesion"]["token"]
    assert sesion.token not in repr(sesion)


def test_schema_de_registro_identifica_contrasena_como_secreta() -> None:
    schema = RegistrarUsuarioRequest.model_json_schema()
    assert schema["properties"]["contrasena"]["writeOnly"] is True
    assert schema["properties"]["contrasena"]["format"] == "password"
    assert "Clave de prueba 123" not in json.dumps(schema)


def test_error_de_credenciales_tiene_el_mensaje_exacto_de_la_especificacion() -> None:
    error: ErrorAutenticacionDTO = error_credenciales_invalidas()
    assert error.mensaje == "Correo o contraseña incorrectos."
