# Archivo principal de la aplicacion
# TODO: mover todo esto a una carpeta de endpoints, para tener mayor orden
# y control en el proyecto

from typing import Annotated

from fastapi import Depends, FastAPI

from cuentas_claras.aplicacion.reloj import Reloj
from cuentas_claras.contrato.ping import PingRespuesta
from cuentas_claras.infraestructura.reloj_sistema import RelojSistema

app = FastAPI(
    title="Cuentas Claras - Servidor",
    version="0.1.0",
)


def obtener_reloj() -> Reloj:
    return RelojSistema()


@app.get("/ping", response_model=PingRespuesta)
def ping(reloj: Annotated[Reloj, Depends(obtener_reloj)]) -> PingRespuesta:
    ahora = reloj.ahora().strftime("%Y-%m-%dT%H:%M:%SZ")
    return PingRespuesta(
        estado="ok",
        mensaje="Servidor Cuentas Claras activo",
        version=app.version,
        timestamp=ahora,
    )