import importlib
import os
from collections.abc import Iterator

import pytest

# Configuración exclusiva de pruebas antes de importar la app; sin BD de desarrollo.
os.environ["JWT_SECRET"] = "solo-qa-entrega6-no-usar-en-produccion-32-bytes"
os.environ["JWT_TTL_MINUTES"] = "60"
os.environ["DATABASE_PATH"] = ":memory:"


@pytest.fixture
def escenario():  # type: ignore[no-untyped-def]
    from qa.entrega6.escenario import crear_escenario

    with crear_escenario() as actual:
        yield actual


@pytest.fixture
def social(escenario):  # type: ignore[no-untyped-def]
    from qa.entrega6.rutas import faltan_rutas
    from qa.entrega6.soporte_social import Social

    faltantes = faltan_rutas(escenario.cliente.app)
    if faltantes:
        pytest.fail(f"Backend E6 pendiente: faltan estas rutas reales: {faltantes}")
    referencia = os.environ.get("CUENTAS_CLARAS_QA_SOCIAL_CONTRACT", "")
    modulo, separador, atributo = referencia.partition(":")
    if not separador or not modulo or not atributo:
        pytest.fail(
            "DTOs E6 pendientes de conectar: definir "
            "CUENTAS_CLARAS_QA_SOCIAL_CONTRACT=modulo:formato. "
            "El formato solo traduce JSON de los DTOs reales; no sustituye rutas."
        )
    formato = getattr(importlib.import_module(modulo), atributo)
    return Social(escenario, formato)


@pytest.fixture(autouse=True)
def no_filtrar_secretos(caplog, capsys) -> Iterator[None]:  # type: ignore[no-untyped-def]
    from qa.entrega6.escenario import CLAVE_PRUEBA

    yield
    salida = capsys.readouterr()
    assert CLAVE_PRUEBA not in caplog.text + salida.out + salida.err
