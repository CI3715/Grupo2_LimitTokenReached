"""Inventario contractual de rutas de la sección 2.6 de la especificación E6."""

import re

from fastapi import FastAPI

ID_INEXISTENTE = "00000000-0000-4000-8000-000000000099"

RUTAS = (
    ("GET", "/usuarios"),
    ("POST", "/amistades"),
    ("POST", "/amistades/{id}/respuesta"),
    ("GET", "/amistades"),
    ("DELETE", "/amistades/{id}"),
    ("POST", "/grupos"),
    ("GET", "/grupos"),
    ("POST", "/grupos/{id}/miembros"),
    ("POST", "/grupos/{id}/miembros/yo/respuesta"),
    ("POST", "/grupos/{id}/gastos"),
    ("GET", "/grupos/{id}"),
)


def rutas_montadas(app: FastAPI) -> set[tuple[str, str]]:
    # OpenAPI aplana también APIRouter incluidos de forma diferida por FastAPI.
    return {
        (metodo.upper(), re.sub(r"\{[^}]+\}", "{id}", ruta))
        for ruta, operaciones in app.openapi()["paths"].items()
        for metodo in operaciones
        if metodo in {"get", "post", "put", "patch", "delete", "head", "options"}
    }


def faltan_rutas(app: FastAPI) -> list[tuple[str, str]]:
    return sorted(set(RUTAS) - rutas_montadas(app))
