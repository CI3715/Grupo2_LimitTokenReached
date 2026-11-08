"""Aceptación E5 contra backend real; fuera de testpaths hasta integrar E5.

No certifica pantallas, almacén de secretos del dispositivo ni autonomía de Tauri.
"""

import json
import logging
from datetime import datetime, timedelta
from typing import Any

import pytest
from fastapi.testclient import TestClient

from cuentas_claras.contrato.autenticacion import (
    ErrorAutenticacionDTO,
    SesionDTO,
    UsuarioDTO,
)
from qa.entrega5.soporte_auth import EscenarioAuth


def registrar_y_entrar(
    cliente: TestClient, datos: dict[str, Any]
) -> tuple[dict[str, Any], SesionDTO]:
    registro = cliente.post("/usuarios", json=datos)
    assert registro.status_code == 201, registro.status_code
    respuesta = cliente.post(
        "/sesiones", json={"correo": datos["correo"], "contrasena": datos["contrasena"]}
    )
    assert respuesta.status_code == 200, respuesta.status_code
    return registro.json(), SesionDTO.model_validate_json(respuesta.text)


def test_registro_devuelve_usuario_publico_y_no_filtra_clave(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    caplog: pytest.LogCaptureFixture,
) -> None:
    with caplog.at_level(logging.DEBUG):
        respuesta = cliente_auth.post("/usuarios", json=registro_auth)
    assert respuesta.status_code == 201
    usuario = UsuarioDTO.model_validate_json(respuesta.text)
    assert usuario.nombre_usuario == registro_auth["nombre_usuario"]
    assert usuario.nombre_visible == registro_auth["nombre_visible"]
    assert usuario.correo.casefold() == registro_auth["correo"].casefold()
    assert registro_auth["contrasena"] not in respuesta.text
    assert registro_auth["contrasena"] not in caplog.text


@pytest.mark.parametrize(
    ("campo", "valor"),
    [
        ("nombre_usuario", "ab"),
        ("nombre_usuario", "a" * 31),
        ("nombre_usuario", "Rosa"),
        ("nombre_usuario", "rosa-qa"),
        ("nombre_visible", " \t"),
        ("correo", "correo-invalido"),
        ("contrasena", "a" * 9),
        ("contrasena", ""),
        ("contrasena", None),
    ],
)
def test_registro_invalido_devuelve_400_y_campo_sin_filtrar_entrada(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    campo: str,
    valor: object,
    caplog: pytest.LogCaptureFixture,
    capsys: pytest.CaptureFixture[str],
) -> None:
    datos = dict(registro_auth, **{campo: valor})
    with caplog.at_level(logging.DEBUG):
        respuesta = cliente_auth.post("/usuarios", json=datos)
    assert respuesta.status_code == 400
    error = ErrorAutenticacionDTO.model_validate_json(respuesta.text)
    assert error.campo == campo
    assert error.mensaje
    salida = capsys.readouterr()
    salida_total = respuesta.text + caplog.text + salida.out + salida.err
    clave = datos.get("contrasena")
    if isinstance(clave, str) and clave.strip():
        assert clave not in salida_total
    assert "Traceback" not in respuesta.text


@pytest.mark.parametrize(
    "campo", ["nombre_usuario", "nombre_visible", "correo", "contrasena"]
)
def test_registro_sin_campo_devuelve_400_en_vez_de_422(
    cliente_auth: TestClient, registro_auth: dict[str, Any], campo: str
) -> None:
    datos = registro_auth.copy()
    del datos[campo]
    respuesta = cliente_auth.post("/usuarios", json=datos)
    assert respuesta.status_code == 400
    assert ErrorAutenticacionDTO.model_validate_json(respuesta.text).campo == campo


@pytest.mark.parametrize(
    ("campo", "cambio"),
    [
        ("nombre_usuario", {"correo": "otra.persona@example.com"}),
        ("correo", {"nombre_usuario": "otra.persona"}),
        (
            "correo",
            {"nombre_usuario": "otra.persona", "correo": "USUARIO.QA@EXAMPLE.COM"},
        ),
    ],
)
def test_duplicados_devuelven_409_con_campo(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    campo: str,
    cambio: dict[str, str],
) -> None:
    assert cliente_auth.post("/usuarios", json=registro_auth).status_code == 201
    respuesta = cliente_auth.post("/usuarios", json=dict(registro_auth, **cambio))
    assert respuesta.status_code == 409
    assert ErrorAutenticacionDTO.model_validate_json(respuesta.text).campo == campo


def test_hash_persistido_no_es_texto_plano_y_usa_sal_distinta(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    escenario_auth: EscenarioAuth,
) -> None:
    from argon2 import PasswordHasher
    from bcrypt import checkpw

    assert cliente_auth.post("/usuarios", json=registro_auth).status_code == 201
    otro = dict(registro_auth, nombre_usuario="otro.usuario", correo="otro@example.com")
    assert cliente_auth.post("/usuarios", json=otro).status_code == 201
    hashes = []
    for correo in [registro_auth["correo"], otro["correo"]]:
        persistido = escenario_auth.registro_persistido(correo)
        # Inspeccionar la fila completa detecta también una columna de texto plano
        # adicional, aunque exista otra columna con un hash correcto.
        assert registro_auth["contrasena"] not in json.dumps(persistido, default=str)
        candidatos = [
            valor
            for valor in persistido.values()
            if isinstance(valor, str)
            and valor.startswith(("$argon2", "$2a$", "$2b$", "$2y$"))
        ]
        assert len(candidatos) == 1
        hashes.append(candidatos[0])
    assert hashes[0] != hashes[1]
    sales = []
    for almacenado in hashes:
        if almacenado.startswith("$argon2"):
            assert PasswordHasher().verify(almacenado, registro_auth["contrasena"])
            sales.append(almacenado.split("$")[4])
        else:
            assert almacenado.startswith(("$2a$", "$2b$", "$2y$"))
            assert checkpw(registro_auth["contrasena"].encode(), almacenado.encode())
            sales.append(almacenado[7:29])
    assert sales[0] != sales[1]  # Comparar la sal, no solo el hash completo.


def test_sesion_dura_60_minutos_y_perfil_exige_bearer_valido(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    escenario_auth: EscenarioAuth,
) -> None:
    inicio = escenario_auth.ahora()
    usuario, sesion = registrar_y_entrar(cliente_auth, registro_auth)
    assert sesion.vence_en - inicio == timedelta(minutes=60)
    perfil = cliente_auth.get(
        "/usuarios/yo", headers={"Authorization": f"Bearer {sesion.token}"}
    )
    assert perfil.status_code == 200
    assert UsuarioDTO.model_validate_json(perfil.text).correo == usuario["correo"]
    assert "id" not in perfil.json()
    assert registro_auth["contrasena"] not in perfil.text


def test_login_acepta_correo_sin_distinguir_mayusculas(
    cliente_auth: TestClient, registro_auth: dict[str, Any]
) -> None:
    assert cliente_auth.post("/usuarios", json=registro_auth).status_code == 201
    respuesta = cliente_auth.post(
        "/sesiones",
        json={
            "correo": registro_auth["correo"].upper(),
            "contrasena": registro_auth["contrasena"],
        },
    )
    assert respuesta.status_code == 200
    SesionDTO.model_validate_json(respuesta.text)


@pytest.mark.parametrize("clave_incorrecta", ["xyz99", "ClaveInvalidaSoloQA_123"])
def test_credenciales_incorrectas_no_revelan_si_el_usuario_existe(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    caplog: pytest.LogCaptureFixture,
    clave_incorrecta: str,
    capsys: pytest.CaptureFixture[str],
) -> None:
    assert cliente_auth.post("/usuarios", json=registro_auth).status_code == 201
    cuerpos = []
    with caplog.at_level(logging.DEBUG):
        for correo in [registro_auth["correo"], "inexistente@example.com"]:
            respuesta = cliente_auth.post(
                "/sesiones", json={"correo": correo, "contrasena": clave_incorrecta}
            )
            assert respuesta.status_code == 401
            error = ErrorAutenticacionDTO.model_validate_json(respuesta.text)
            assert error.mensaje == "Correo o contraseña incorrectos."
            cuerpos.append(respuesta.json())
            assert clave_incorrecta not in respuesta.text
    assert cuerpos[0] == cuerpos[1]
    salida = capsys.readouterr()
    assert clave_incorrecta not in caplog.text + salida.out + salida.err
    assert registro_auth["contrasena"] not in caplog.text + salida.out + salida.err


@pytest.mark.parametrize(
    "cabecera",
    [
        {},
        {"Authorization": "Bearer token-invalido"},
        {"Authorization": "Basic token-invalido"},
    ],
)
def test_perfil_rechaza_token_ausente_o_invalido_con_401(
    cliente_auth: TestClient, cabecera: dict[str, str]
) -> None:
    assert cliente_auth.get("/usuarios/yo", headers=cabecera).status_code == 401


def test_perfil_rechaza_token_alterado_y_vencido(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    escenario_auth: EscenarioAuth,
) -> None:
    _, sesion = registrar_y_entrar(cliente_auth, registro_auth)
    # Cambiar la cabecera invalida un token firmado sin depender del tipo de firma.
    alterado = ("B" if sesion.token[0] != "B" else "C") + sesion.token[1:]
    assert (
        cliente_auth.get(
            "/usuarios/yo", headers={"Authorization": f"Bearer {alterado}"}
        ).status_code
        == 401
    )
    escenario_auth.avanzar_reloj(timedelta(minutes=59, seconds=59))
    assert (
        cliente_auth.get(
            "/usuarios/yo", headers={"Authorization": f"Bearer {sesion.token}"}
        ).status_code
        == 200
    )
    escenario_auth.avanzar_reloj(timedelta(seconds=1))
    assert (
        cliente_auth.get(
            "/usuarios/yo", headers={"Authorization": f"Bearer {sesion.token}"}
        ).status_code
        == 401
    )


def test_registro_no_admite_datos_financieros(
    cliente_auth: TestClient, registro_auth: dict[str, Any]
) -> None:
    respuesta = cliente_auth.post(
        "/usuarios",
        json=dict(registro_auth, cuentas=[], movimientos=[], tasas=[], saldos=[]),
    )
    assert respuesta.status_code == 400


def test_fecha_alta_del_usuario_usa_reloj_del_escenario(
    cliente_auth: TestClient,
    registro_auth: dict[str, Any],
    escenario_auth: EscenarioAuth,
) -> None:
    ahora: datetime = escenario_auth.ahora()
    respuesta = cliente_auth.post("/usuarios", json=registro_auth)
    assert respuesta.status_code == 201
    assert UsuarioDTO.model_validate_json(respuesta.text).fecha_alta == ahora
