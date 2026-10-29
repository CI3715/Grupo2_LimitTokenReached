from datetime import datetime, timezone

from fastapi.testclient import TestClient

from cuentas_claras.app.main import app, obtener_reloj


class RelojFijo:

    def ahora(self) -> datetime:
        return datetime(2026, 10, 8, 10, 0, 0, tzinfo=timezone.utc)


def test_ping_devuelve_el_contrato() -> None:
    app.dependency_overrides[obtener_reloj] = RelojFijo
    try:
        respuesta = TestClient(app).get("/ping")
    finally:
        app.dependency_overrides.clear()

    assert respuesta.status_code == 200
    assert respuesta.json() == {
        "estado": "ok",
        "mensaje": "Servidor Cuentas Claras activo",
        "version": "0.1.0",
        "timestamp": "2026-10-08T10:00:00Z",
    }
