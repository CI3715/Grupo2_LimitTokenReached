from cuentas_claras.app.main import app
from fastapi.testclient import TestClient


def test_documentacion_disponible() -> None:
    cliente = TestClient(app)
    respuesta = cliente.get("/docs")
    assert respuesta.status_code == 200
