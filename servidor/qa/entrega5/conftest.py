import importlib
import json
import logging
import os
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest
from fastapi.testclient import TestClient

from qa.entrega5.soporte_auth import EscenarioAuth


@pytest.fixture
def escenario_auth() -> Iterator[EscenarioAuth]:
    referencia = os.environ.get("CUENTAS_CLARAS_QA_AUTH_FACTORY", "")
    modulo, separador, nombre = referencia.partition(":")
    if not separador or not modulo or not nombre:
        pytest.fail(
            "Backend E5 pendiente de conectar: definir "
            "CUENTAS_CLARAS_QA_AUTH_FACTORY=modulo:crear_escenario. "
            "La fábrica debe usar la app real, BD de pruebas y reloj controlado."
        )
    fabrica = getattr(importlib.import_module(modulo), nombre)
    escenario = fabrica()
    if not isinstance(escenario, EscenarioAuth):
        pytest.fail("La fábrica debe devolver un EscenarioAuth de soporte_auth.")
    try:
        yield escenario
    finally:
        escenario.cerrar()


@pytest.fixture
def cliente_auth(escenario_auth: EscenarioAuth) -> Iterator[TestClient]:
    with TestClient(escenario_auth.app) as cliente:
        yield cliente


@pytest.fixture
def registro_auth() -> dict[str, Any]:
    ruta = Path(__file__).resolve().parents[3] / "qa/fixtures/autenticacion.json"
    return json.loads(ruta.read_text(encoding="utf-8"))["registro"]


@pytest.fixture(autouse=True)
def no_filtrar_clave_en_logs(
    registro_auth: dict[str, Any],
    caplog: pytest.LogCaptureFixture,
    capsys: pytest.CaptureFixture[str],
) -> Iterator[None]:
    caplog.set_level(logging.DEBUG)
    yield
    salida = capsys.readouterr()
    assert registro_auth["contrasena"] not in caplog.text + salida.out + salida.err
