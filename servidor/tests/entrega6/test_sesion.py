"""Regresión de la sesión usada por E6, con autorización y repositorio reales."""

from datetime import timedelta
from typing import Any

import jwt
import pytest
from qa.entrega6.escenario import CLAVE_PRUEBA, USUARIO_IDS, EscenarioSocial
from qa.entrega6.rutas import rutas_montadas
from sqlalchemy import delete

from cuentas_claras.app.config.config import settings
from cuentas_claras.infraestructura.modelos_usuario import UsuarioDB


@pytest.mark.parametrize(
    "autorizacion",
    [None, "", "Bearer", "Bearer ", "Basic abc", "Token abc", "Bearer abc.def.ghi"],
)
def test_perfil_rechaza_sesion_ausente_o_malformada(
    escenario: EscenarioSocial, autorizacion: str | None
) -> None:
    headers = {} if autorizacion is None else {"Authorization": autorizacion}
    respuesta = escenario.cliente.get("/usuarios/yo", headers=headers)
    assert respuesta.status_code == 401
    assert respuesta.headers["www-authenticate"] == "Bearer"
    assert "correo" not in respuesta.json()


def _firmar(escenario: EscenarioSocial, **cambios: Any) -> str:
    datos = {
        "sub": USUARIO_IDS["ana"],
        "iss": settings.JWT_ISSUER,
        "iat": escenario.reloj.ahora(),
        "exp": escenario.reloj.ahora() + timedelta(minutes=60),
    }
    datos.update(cambios)
    return jwt.encode(datos, settings.JWT_SECRET.get_secret_value(), algorithm="HS256")


@pytest.mark.parametrize(
    "cambios",
    [
        {"sub": "desconocido"},
        {"sub": ""},
        {"sub": 123},
        {"iss": "otro-emisor"},
        {"exp": None},
        {"exp": "mañana"},
        {"exp": True},
        {"exp": 10**100},
        {"exp": 0},
    ],
)
def test_claims_invalidos_no_abren_la_sesion(
    escenario: EscenarioSocial, cambios: dict[str, Any]
) -> None:
    token = _firmar(escenario, **cambios)
    respuesta = escenario.cliente.get(
        "/usuarios/yo", headers={"Authorization": f"Bearer {token}"}
    )
    assert respuesta.status_code == 401
    assert token not in respuesta.text


@pytest.mark.parametrize("claim", ["sub", "exp", "iss"])
def test_claim_obligatorio_ausente(escenario: EscenarioSocial, claim: str) -> None:
    datos = jwt.decode(_firmar(escenario), options={"verify_signature": False})
    del datos[claim]
    token = jwt.encode(datos, settings.JWT_SECRET.get_secret_value(), algorithm="HS256")
    respuesta = escenario.cliente.get(
        "/usuarios/yo", headers={"Authorization": f"Bearer {token}"}
    )
    assert respuesta.status_code == 401


def test_firma_ajena_y_token_sin_firma_no_son_sesion(
    escenario: EscenarioSocial,
) -> None:
    datos = jwt.decode(_firmar(escenario), options={"verify_signature": False})
    for token in (
        jwt.encode(datos, "otra-clave-exclusiva-de-prueba-32-bytes", algorithm="HS256"),
        jwt.encode(datos, "", algorithm="none"),
    ):
        respuesta = escenario.cliente.get(
            "/usuarios/yo", headers={"Authorization": f"Bearer {token}"}
        )
        assert respuesta.status_code == 401


def test_token_valido_solo_identifica_a_su_titular(
    escenario: EscenarioSocial,
) -> None:
    for usuario in ("ana", "beto", "carla", "diana"):
        respuesta = escenario.cliente.get(
            "/usuarios/yo", headers=escenario.cabecera(usuario)
        )
        assert respuesta.status_code == 200
        assert respuesta.json()["nombre_usuario"] == usuario
        assert respuesta.json()["correo"] == f"{usuario}@example.com"
        assert set(respuesta.json()) == {
            "nombre_usuario",
            "nombre_visible",
            "correo",
            "fecha_alta",
        }
        assert CLAVE_PRUEBA not in respuesta.text
        assert "hash_contrasena" not in respuesta.text


def test_login_real_emite_token_utilizable_y_caduca_en_el_limite(
    escenario: EscenarioSocial,
) -> None:
    respuesta = escenario.cliente.post(
        "/sesiones", json={"correo": "ANA@example.com", "contrasena": CLAVE_PRUEBA}
    )
    assert respuesta.status_code == 200
    token = respuesta.json()["token"]
    datos = jwt.decode(token, options={"verify_signature": False})
    assert datos["exp"] - datos["iat"] == 3600
    headers = {"Authorization": f"Bearer {token}"}
    assert escenario.cliente.get("/usuarios/yo", headers=headers).status_code == 200
    escenario.reloj.avanzar(3599)
    assert escenario.cliente.get("/usuarios/yo", headers=headers).status_code == 200
    escenario.reloj.avanzar(1)
    respuesta = escenario.cliente.get("/usuarios/yo", headers=headers)
    assert respuesta.status_code == 401
    assert respuesta.headers["www-authenticate"] == "Bearer"


def test_token_de_usuario_eliminado_no_da_acceso(escenario: EscenarioSocial) -> None:
    with escenario.fabrica_db() as db:
        db.execute(delete(UsuarioDB).where(UsuarioDB.nombre_usuario == "ana"))
        db.commit()
    respuesta = escenario.cliente.get("/usuarios/yo", headers=escenario.cabecera("ana"))
    assert respuesta.status_code == 401


def test_consulta_no_admite_token_en_query(escenario: EscenarioSocial) -> None:
    respuesta = escenario.cliente.get(
        "/usuarios/yo", params={"token": escenario.tokens["ana"]}
    )
    assert respuesta.status_code == 401


def test_token_no_se_puede_usar_como_clave_de_login(
    escenario: EscenarioSocial,
) -> None:
    respuesta = escenario.cliente.post(
        "/sesiones",
        json={"correo": "ana@example.com", "contrasena": escenario.tokens["ana"]},
    )
    assert respuesta.status_code == 401
    assert respuesta.json()["mensaje"] == "Correo o contraseña incorrectos."


@pytest.mark.parametrize("ruta", ["/usuarios", "/sesiones"])
def test_errores_de_entrada_son_400_sin_filtrar_clave(
    escenario: EscenarioSocial, ruta: str
) -> None:
    respuesta = escenario.cliente.post(
        ruta, json={"correo": "correo-invalido", "contrasena": CLAVE_PRUEBA}
    )
    assert respuesta.status_code == 400
    assert CLAVE_PRUEBA not in respuesta.text
    assert set(respuesta.json()) == {"campo", "mensaje"}


def test_ping_sigue_disponible_sin_sesion(escenario: EscenarioSocial) -> None:
    assert escenario.cliente.get("/ping").status_code == 200


def test_inventario_detecta_rutas_reales_incluidas_por_router(
    escenario: EscenarioSocial,
) -> None:
    assert {
        ("POST", "/usuarios"),
        ("POST", "/sesiones"),
        ("GET", "/usuarios/yo"),
        ("GET", "/ping"),
    } <= rutas_montadas(escenario.cliente.app)
