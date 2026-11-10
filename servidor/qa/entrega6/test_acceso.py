"""Ejecutar: python -m pytest qa/entrega6/test_acceso.py -q.

No se omiten rutas ausentes: un 404 donde se exige 401 es un fallo de integración.
No hace falta conectar los DTOs para ejecutar esta matriz.
"""

from datetime import timedelta

import jwt
import pytest

from qa.entrega6.rutas import ID_INEXISTENTE, RUTAS, faltan_rutas


def test_todas_las_rutas_estan_montadas_en_la_app_real(escenario):
    assert not faltan_rutas(escenario.cliente.app), faltan_rutas(escenario.cliente.app)


@pytest.mark.parametrize(("metodo", "ruta"), RUTAS)
@pytest.mark.parametrize(
    "sesion", ["ausente", "malformada", "firma_ajena", "vencida", "sin_firma"]
)
def test_cada_endpoint_requiere_sesion_valida(escenario, metodo, ruta, sesion):
    from cuentas_claras.app.config.config import settings
    from qa.entrega6.escenario import USUARIO_IDS

    headers = {}
    if sesion == "malformada":
        headers["Authorization"] = "Bearer token-invalido"
    elif sesion != "ausente":
        ahora = escenario.reloj.ahora()
        datos = {
            "sub": USUARIO_IDS["ana"],
            "iss": settings.JWT_ISSUER,
            "iat": ahora - timedelta(minutes=61),
            "exp": (
                ahora - timedelta(minutes=1)
                if sesion == "vencida"
                else ahora + timedelta(minutes=60)
            ),
        }
        clave = (
            "clave-ajena-exclusiva-qa-con-32-bytes"
            if sesion == "firma_ajena"
            else settings.JWT_SECRET.get_secret_value()
        )
        token = jwt.encode(
            datos,
            "" if sesion == "sin_firma" else clave,
            algorithm="none" if sesion == "sin_firma" else "HS256",
        )
        headers["Authorization"] = f"Bearer {token}"
    url = ruta.replace("{id}", ID_INEXISTENTE)
    if ruta == "/usuarios":
        url += "?usuario=beto"
    respuesta = escenario.cliente.request(metodo, url, headers=headers)
    assert respuesta.status_code == 401, (
        f"{metodo} {ruta}, sesión {sesion}: se esperaba 401; "
        f"se recibió {respuesta.status_code}. Revisar montaje y autorización."
    )


@pytest.mark.parametrize(("metodo", "ruta"), RUTAS)
def test_token_en_query_no_sustituye_bearer(escenario, metodo, ruta):
    url = ruta.replace("{id}", ID_INEXISTENTE)
    respuesta = escenario.cliente.request(
        metodo,
        url,
        params={"usuario": "beto", "token": escenario.tokens["ana"]},
    )
    assert respuesta.status_code == 401
