"""Traductor de las pruebas E6 hacia los JSON reales de la API social.

Solo renombra campos y ordena envolturas. No calcula resultados ni inventa
ids, estados, gastos o balances: todo lo que devuelve viene de la API.
"""

from datetime import date
from typing import Any

CAMPOS: dict[str, tuple[str, ...]] = {
    "amistad": ("usuario",),
    "respuesta_amistad": ("aceptar",),
    "miembro": ("usuario",),
    "respuesta_grupo": ("aceptar",),
    "grupo": ("nombre", "descripcion", "moneda", "actividad"),
    "gasto": (
        "concepto",
        "monto",
        "fecha",
        "pagador",
        "regla",
        "participantes",
        "valores",
    ),
}


class FormatoReal:
    def datos(self, operacion: str, **valores: Any) -> dict[str, Any]:
        if operacion not in CAMPOS:
            raise ValueError(f"Operación desconocida: {operacion}")
        resultado = {c: valores[c] for c in CAMPOS[operacion] if c in valores}
        fecha = resultado.get("fecha")
        if isinstance(fecha, date):
            resultado["fecha"] = fecha.isoformat()
        return resultado

    def leer(self, operacion: str, respuesta: Any) -> dict[str, Any]:
        cuerpo = respuesta.json() if hasattr(respuesta, "json") else respuesta
        if operacion in ("amistad", "grupo"):
            return {"id": cuerpo["id"]}
        if operacion == "amistades":
            return {
                clave: [elemento["nombre_usuario"] for elemento in cuerpo[clave]]
                for clave in ("amigos", "enviadas", "recibidas")
            }
        if operacion == "grupos":
            return {
                clave: [grupo["id"] for grupo in cuerpo[clave]]
                for clave in ("miembros", "invitaciones")
            }
        return dict(cuerpo)


FORMATO = FormatoReal()
