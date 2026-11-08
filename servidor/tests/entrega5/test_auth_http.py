import json
import logging
from collections.abc import Iterator
from datetime import datetime, timedelta, timezone
from pathlib import Path

import jwt
import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine, select
from sqlalchemy.engine import Engine
from sqlalchemy.orm import Session, sessionmaker
from sqlalchemy.pool import StaticPool

from cuentas_claras.app.main import app
from cuentas_claras.contrato.autenticacion import UsuarioDTO
from cuentas_claras.infraestructura.base_datos import Base, obtener_db
from cuentas_claras.infraestructura.configuracion import settings
from cuentas_claras.infraestructura.modelos_usuario import UsuarioDB


@pytest.fixture
def engine_prueba(tmp_path: Path) -> Iterator[Engine]:
    ruta = tmp_path / "auth-test.db"
    engine = create_engine(
        f"sqlite:///{ruta.as_posix()}",
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )
    Base.metadata.create_all(bind=engine)
    yield engine
    Base.metadata.drop_all(bind=engine)
    engine.dispose()


@pytest.fixture
def cliente(engine_prueba: Engine) -> Iterator[TestClient]:
    fabrica = sessionmaker(bind=engine_prueba, autoflush=False, expire_on_commit=False)

    def db_prueba() -> Iterator[Session]:
        sesion = fabrica()
        try:
            yield sesion
        finally:
            sesion.close()

    app.dependency_overrides[obtener_db] = db_prueba
    try:
        with TestClient(app) as test_client:
            yield test_client
    finally:
        app.dependency_overrides.clear()


@pytest.fixture
def registro() -> dict[str, str]:
    return {
        "nombre_usuario": "usuario.qa",
        "nombre_visible": "Usuario QA",
        "correo": "usuario.qa@example.com",
        "contrasena": "ClaveSeguraDePrueba123",
    }


def test_registro_persiste_hash_y_responde_solo_usuario_publico(
    cliente: TestClient,
    registro: dict[str, str],
    engine_prueba: Engine,
    caplog: pytest.LogCaptureFixture,
) -> None:
    with caplog.at_level(logging.DEBUG):
        respuesta = cliente.post("/usuarios", json=registro)

    assert respuesta.status_code == 201
    dto = UsuarioDTO.model_validate(respuesta.json())
    assert dto.nombre_usuario == registro["nombre_usuario"]
    assert "hash_contrasena" not in respuesta.text
    assert registro["contrasena"] not in respuesta.text
    assert registro["contrasena"] not in caplog.text

    segundo = {
        **registro,
        "nombre_usuario": "otro.usuario",
        "correo": "otro.usuario@example.com",
    }
    assert cliente.post("/usuarios", json=segundo).status_code == 201
    with Session(engine_prueba) as db:
        filas = list(db.scalars(select(UsuarioDB).order_by(UsuarioDB.correo)))
        hashes = [fila.hash_contrasena for fila in filas]
        assert len(hashes) == 2
        assert all(hash_guardado.startswith("$argon2") for hash_guardado in hashes)
        assert hashes[0] != hashes[1]
        assert all(
            registro["contrasena"]
            not in json.dumps(
                {
                    columna.name: getattr(fila, columna.name)
                    for columna in UsuarioDB.__table__.columns
                },
                default=str,
            )
            for fila in filas
        )


@pytest.mark.parametrize(
    ("cambio", "campo"),
    [
        ({"nombre_usuario": "usuario.qa"}, "nombre_usuario"),
        (
            {
                "nombre_usuario": "otro.usuario",
                "correo": "USUARIO.QA@EXAMPLE.COM",
            },
            "correo",
        ),
    ],
)
def test_duplicados_indican_el_campo(
    cliente: TestClient,
    registro: dict[str, str],
    cambio: dict[str, str],
    campo: str,
) -> None:
    assert cliente.post("/usuarios", json=registro).status_code == 201
    respuesta = cliente.post("/usuarios", json={**registro, **cambio})

    assert respuesta.status_code == 409
    assert respuesta.json()["campo"] == campo


@pytest.mark.parametrize(
    ("cambio", "campo"),
    [
        ({"correo": "no-es-correo"}, "correo"),
        ({"contrasena": "corta"}, "contrasena"),
    ],
)
def test_datos_invalidos_responden_400(
    cliente: TestClient,
    registro: dict[str, str],
    cambio: dict[str, str],
    campo: str,
) -> None:
    respuesta = cliente.post("/usuarios", json={**registro, **cambio})
    assert respuesta.status_code == 400
    assert respuesta.json()["campo"] == campo


def test_login_usa_mismo_error_para_cuenta_existente_o_inexistente(
    cliente: TestClient, registro: dict[str, str]
) -> None:
    cliente.post("/usuarios", json=registro)
    cuerpos = []
    for correo in [registro["correo"], "noexiste@example.com"]:
        respuesta = cliente.post(
            "/sesiones",
            json={"correo": correo, "contrasena": "ClaveIncorrecta123"},
        )
        assert respuesta.status_code == 401
        cuerpos.append(respuesta.json())

    assert cuerpos[0] == cuerpos[1]
    assert cuerpos[0]["mensaje"] == "Correo o contraseña incorrectos."


def test_login_ignora_mayusculas_del_correo_y_entrega_sesion_de_60_minutos(
    cliente: TestClient, registro: dict[str, str]
) -> None:
    cliente.post("/usuarios", json=registro)
    antes = datetime.now(timezone.utc)
    respuesta = cliente.post(
        "/sesiones",
        json={
            "correo": registro["correo"].upper(),
            "contrasena": registro["contrasena"],
        },
    )
    vence = datetime.fromisoformat(respuesta.json()["vence_en"])

    assert respuesta.status_code == 200
    assert timedelta(minutes=60) <= vence - antes <= timedelta(minutes=60, seconds=2)


def test_token_permite_perfil_y_perfil_rechaza_token_invalido(
    cliente: TestClient, registro: dict[str, str]
) -> None:
    cliente.post("/usuarios", json=registro)
    inicio = cliente.post(
        "/sesiones",
        json={
            "correo": registro["correo"],
            "contrasena": registro["contrasena"],
        },
    )
    token = inicio.json()["token"]

    perfil = cliente.get("/usuarios/yo", headers={"Authorization": f"Bearer {token}"})
    assert perfil.status_code == 200
    assert perfil.json()["nombre_visible"] == registro["nombre_visible"]
    assert cliente.get("/usuarios/yo").status_code == 401
    assert (
        cliente.get(
            "/usuarios/yo",
            headers={"Authorization": "Bearer token-alterado"},
        ).status_code
        == 401
    )


def test_registro_rechaza_cualquier_campo_financiero(
    cliente: TestClient, registro: dict[str, str]
) -> None:
    respuesta = cliente.post(
        "/usuarios",
        json={**registro, "cuentas": [], "movimientos": [], "saldos": [], "tasas": []},
    )
    assert respuesta.status_code == 400


def test_perfil_rechaza_token_vencido(cliente: TestClient) -> None:
    token = jwt.encode(
        {
            "sub": "usuario-que-no-existe",
            "iss": settings.JWT_ISSUER,
            "iat": datetime.now(timezone.utc) - timedelta(minutes=2),
            "exp": datetime.now(timezone.utc) - timedelta(minutes=1),
        },
        settings.JWT_SECRET.get_secret_value(),
        algorithm="HS256",
    )
    respuesta = cliente.get(
        "/usuarios/yo", headers={"Authorization": f"Bearer {token}"}
    )
    assert respuesta.status_code == 401
