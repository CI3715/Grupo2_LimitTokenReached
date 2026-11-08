from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Annotated

from fastapi import Depends, FastAPI, Request
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse

from cuentas_claras.aplicacion.reloj import Reloj
from cuentas_claras.app.rutas.autenticacion import router as auth_router
from cuentas_claras.contrato.autenticacion import error_datos_invalidos
from cuentas_claras.contrato.ping import PingRespuesta
from cuentas_claras.infraestructura.base_datos import Base, engine
from cuentas_claras.infraestructura.reloj_sistema import RelojSistema


@asynccontextmanager
async def lifespan(_: FastAPI) -> AsyncIterator[None]:
    Base.metadata.create_all(bind=engine)
    yield


app = FastAPI(
    title="Cuentas Claras - Servidor",
    version="0.1.0",
    lifespan=lifespan,
)
app.include_router(auth_router)


@app.exception_handler(RequestValidationError)
async def manejar_datos_invalidos(
    _: Request, error: RequestValidationError
) -> JSONResponse:
    dto = error_datos_invalidos(error.errors())
    return JSONResponse(status_code=400, content=dto.model_dump())


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
