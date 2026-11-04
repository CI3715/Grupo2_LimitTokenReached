from typing import Any

import pytest
from pydantic import ValidationError

from cuentas_claras.aplicacion.validar_registro_usuario import validar_unicidad_usuario
from cuentas_claras.contrato.autenticacion import (
    RegistrarUsuarioRequest,
    error_datos_invalidos,
)
from cuentas_claras.dominio.validacion_usuario import ErrorValidacionUsuario


@pytest.mark.parametrize("nombre", ["abc", "a" * 30, "rosa_123.qa", "123", "..."])
def test_registro_acepta_limites_y_caracteres_permitidos(
    contratos_auth: dict[str, Any], nombre: str
) -> None:
    datos = dict(contratos_auth["registro"], nombre_usuario=nombre)
    assert RegistrarUsuarioRequest.model_validate(datos).nombre_usuario == nombre


@pytest.mark.parametrize(
    ("campo", "valor"),
    [
        ("nombre_usuario", ""),
        ("nombre_usuario", "ab"),
        ("nombre_usuario", "a" * 31),
        ("nombre_usuario", "Rosa"),
        ("nombre_usuario", "rosa-qa"),
        ("nombre_usuario", "rosa qa"),
        ("nombre_usuario", " rosa "),
        ("nombre_usuario", "rosá"),
        ("nombre_usuario", "rosa\n"),
        ("nombre_visible", ""),
        ("nombre_visible", " \t\n"),
        ("correo", ""),
        ("correo", "correo-sin-arroba"),
        ("correo", "@example.com"),
        ("correo", "usuario@"),
        ("correo", "usuario..qa@example.com"),
        ("correo", "usuario@-example.com"),
        ("correo", "usuario qa@example.com"),
        ("contrasena", ""),
        ("contrasena", " " * 10),
        ("contrasena", "a" * 9),
    ],
)
def test_registro_rechaza_dato_invalido_indicando_el_campo(
    contratos_auth: dict[str, Any], campo: str, valor: str
) -> None:
    datos = dict(contratos_auth["registro"], **{campo: valor})
    with pytest.raises(ValidationError) as resultado:
        RegistrarUsuarioRequest.model_validate(datos)
    error = error_datos_invalidos(resultado.value.errors())
    assert error.campo == campo
    assert error.mensaje
    assert "Traceback" not in error.mensaje


@pytest.mark.parametrize(
    "campo", ["nombre_usuario", "nombre_visible", "correo", "contrasena"]
)
@pytest.mark.parametrize("valor", [None, 123, ["texto"]])
def test_registro_no_coacciona_campos_de_otro_tipo(
    contratos_auth: dict[str, Any], campo: str, valor: object
) -> None:
    with pytest.raises(ValidationError) as resultado:
        RegistrarUsuarioRequest.model_validate(
            dict(contratos_auth["registro"], **{campo: valor})
        )
    assert error_datos_invalidos(resultado.value.errors()).campo == campo


@pytest.mark.parametrize(
    "campo", ["nombre_usuario", "nombre_visible", "correo", "contrasena"]
)
def test_registro_rechaza_cada_campo_ausente(
    contratos_auth: dict[str, Any], campo: str
) -> None:
    datos = contratos_auth["registro"].copy()
    del datos[campo]
    with pytest.raises(ValidationError) as resultado:
        RegistrarUsuarioRequest.model_validate(datos)
    error = error_datos_invalidos(resultado.value.errors())
    assert error.campo == campo
    assert error.mensaje == "El campo es obligatorio."


@pytest.mark.parametrize("clave", ["a" * 10, "á" * 10, "  abcdefgh  "])
def test_registro_acepta_minimo_sin_imponer_complejidad_ni_recortar_clave(
    contratos_auth: dict[str, Any], clave: str
) -> None:
    datos = dict(contratos_auth["registro"], contrasena=clave)
    request = RegistrarUsuarioRequest.model_validate(datos)
    assert request.contrasena.get_secret_value() == clave


def test_normaliza_correo_y_nombre_visible_sin_dns(
    contratos_auth: dict[str, Any], monkeypatch: pytest.MonkeyPatch
) -> None:
    def prohibir_red(*args: object, **kwargs: object) -> None:
        pytest.fail("Validar formato no debe consultar DNS ni abrir conexiones")

    monkeypatch.setattr("socket.getaddrinfo", prohibir_red)
    monkeypatch.setattr("socket.create_connection", prohibir_red)
    request = RegistrarUsuarioRequest.model_validate(
        dict(
            contratos_auth["registro"],
            correo="  USUARIO.QA+Prueba@EXAMPLE.COM  ",
            nombre_visible="  Rosa de prueba  ",
        )
    )
    assert request.correo == "usuario.qa+prueba@example.com"
    assert request.nombre_visible == "Rosa de prueba"


class ConsultaUsuariosMemoria:
    def __init__(self) -> None:
        self.registros = [("usuario.qa", "usuario.qa@example.com")]
        self.consultas_correo: list[str] = []

    def existe_nombre_usuario(self, nombre_usuario: str) -> bool:
        return any(nombre == nombre_usuario for nombre, _ in self.registros)

    def existe_correo(self, correo_normalizado: str) -> bool:
        self.consultas_correo.append(correo_normalizado)
        return any(correo == correo_normalizado for _, correo in self.registros)


@pytest.mark.parametrize(
    ("nombre", "correo", "campo"),
    [
        ("usuario.qa", "otro@example.com", "nombre_usuario"),
        ("otro.usuario", "usuario.qa@example.com", "correo"),
        ("otro.usuario", "USUARIO.QA@EXAMPLE.COM", "correo"),
        ("otro.usuario", "Usuario.Qa@Example.Com", "correo"),
    ],
)
def test_consulta_repositorio_real_del_validador_para_detectar_duplicados(
    nombre: str, correo: str, campo: str
) -> None:
    usuarios = ConsultaUsuariosMemoria()
    with pytest.raises(ErrorValidacionUsuario) as resultado:
        validar_unicidad_usuario(nombre, correo, usuarios)
    assert resultado.value.campo == campo
    assert resultado.value.codigo.endswith("duplicado")
    assert len(usuarios.registros) == 1
    if campo == "correo":
        assert usuarios.consultas_correo == ["usuario.qa@example.com"]


def test_usuario_y_correo_nuevos_superan_la_consulta_de_unicidad() -> None:
    usuarios = ConsultaUsuariosMemoria()
    validar_unicidad_usuario("nuevo.usuario", "NUEVO@EXAMPLE.COM", usuarios)
    assert usuarios.consultas_correo == ["nuevo@example.com"]
    assert len(usuarios.registros) == 1  # Este validador no registra usuarios.
