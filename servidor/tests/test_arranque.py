from fastapi.testclient import TestClient

from cuentas_claras.app.main import app


def test_documentacion_disponible() -> None:
    cliente = TestClient(app)
    respuesta = cliente.get("/docs")
    assert respuesta.status_code == 200
