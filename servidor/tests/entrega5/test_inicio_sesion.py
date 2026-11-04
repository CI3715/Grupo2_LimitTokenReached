from typing import Any

import pytest
from pydantic import ValidationError

from cuentas_claras.contrato.autenticacion import (
    IniciarSesionRequest,
    error_datos_invalidos,
)


@pytest.mark.parametrize(
    "clave", ["corta", "clave-de-prueba-123", "  clave con espacios  "]
)
def test_inicio_delega_la_verificacion_de_clave_al_autenticador(
    contratos_auth: dict[str, Any], clave: str
) -> None:
    datos = dict(contratos_auth["inicio_sesion"], contrasena=clave)
    assert (
        IniciarSesionRequest.model_validate(datos).contrasena.get_secret_value()
        == clave
    )


@pytest.mark.parametrize(
    ("campo", "valor"),
    [
        ("correo", ""),
        ("correo", "correo-invalido"),
        ("contrasena", ""),
        ("contrasena", " " * 12),
        ("correo", None),
        ("contrasena", 123),
    ],
)
def test_inicio_rechaza_datos_ausentes_o_invalidos(
    contratos_auth: dict[str, Any], campo: str, valor: object
) -> None:
    with pytest.raises(ValidationError) as resultado:
        IniciarSesionRequest.model_validate(
            dict(contratos_auth["inicio_sesion"], **{campo: valor})
        )
    assert error_datos_invalidos(resultado.value.errors()).campo == campo


def test_inicio_normaliza_el_correo_sin_tocar_la_clave(
    contratos_auth: dict[str, Any],
) -> None:
    datos = dict(contratos_auth["inicio_sesion"], correo=" USUARIO.QA@EXAMPLE.COM ")
    request = IniciarSesionRequest.model_validate(datos)
    assert request.correo == "usuario.qa@example.com"
    assert request.contrasena.get_secret_value() == datos["contrasena"]
