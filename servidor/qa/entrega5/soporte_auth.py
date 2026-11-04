"""Punto de conexión de QA con la implementación real del backend E5.

Ejecutar desde servidor, con el extra qa-auth instalado:
CUENTAS_CLARAS_QA_AUTH_FACTORY=modulo_de_pruebas:crear_escenario \
    python -m pytest qa/entrega5

Importar EscenarioAuth desde qa.entrega5.soporte_auth en la fábrica.
La fábrica del equipo backend debe devolver EscenarioAuth usando su FastAPI real,
una BD de pruebas nueva por escenario y su reloj inyectable.
registro_persistido debe leer todas las columnas del usuario desde esa BD,
sin filtrar contraseñas ni preparar un hash solo para hacer pasar el test.
No se proporciona un servidor simulado que pueda hacer pasar esta suite.
"""

from collections.abc import Callable, Mapping
from dataclasses import dataclass
from datetime import datetime, timedelta

from fastapi import FastAPI


@dataclass(frozen=True)
class EscenarioAuth:
    app: FastAPI
    ahora: Callable[[], datetime]
    avanzar_reloj: Callable[[timedelta], None]
    registro_persistido: Callable[[str], Mapping[str, object]]
    cerrar: Callable[[], None]
