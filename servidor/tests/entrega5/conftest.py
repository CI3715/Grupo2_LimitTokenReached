import json
from pathlib import Path
from typing import Any

import pytest


@pytest.fixture
def contratos_auth() -> dict[str, Any]:
    ruta = Path(__file__).resolve().parents[3] / "qa/fixtures/autenticacion.json"
    return json.loads(ruta.read_text(encoding="utf-8"))
