# Archivo principal de la aplicacion
# TODO: mover todo esto a una carpeta de endpoints, para tener mayor orden
# y control en el proyecto

from datetime import datetime, timezone

from fastapi import FastAPI, status
from fastapi.responses import JSONResponse

from cuentas_claras.app.config.config import Settings

app = FastAPI(
    title="Cuentas Claras - Servidor",
    version="0.1.0",
)

settings = Settings()


# esta consulta retorna la latencia del servidor y si existe
#  conexion con el servidor de cuentas claras
# la funcion es asincrona ya que debe esperar la
# respuesta del servidor de cuentas claras,
# y no queremos que el servidor se bloquee mientras espera la respuesta

# Dejemoslo aqui por si acaso
# @app.get("/ping")
# async def conection(url: str = settings.SERVER_HOST_IP) -> JSONResponse:
#     """
#     Endpoint que sirve para verificar la conexion con el servidor de cuentas claras
#     y medir la latencia de la conexion.
#     """

#     # esto obtiene la hora actual
#     timestamp_actual = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")

#     # Hay que definir un tiempo inicial, demanera que podamos calcular la latencia

#     # realizamos la consulta al servidor de cuentas claras

#     async with httpx.AsyncClient() as client:
#         # Camino del bien: que la conexion con el server responda
#         try:

#             # vamos a guardar en la variable response la respuesta
#             # que obtuviimos al consultar la url
#             await client.get(url, timeout=5.0)

#             # una vez obtenida la respuesta

#             return JSONResponse(
#                 status_code=status.HTTP_200_OK,
#                 content={
#                     "estado": "ok",
#                     "mensaje": "Servidor de Cuentas Claras activo",
#                     "version": app.version,
#                     "timestamp": timestamp_actual,
#                 },
#             )
#         # camino del mal: la conexion no se pudo establecer,
#         #  de igual manera debemos dar una
#         # respuesta indicando
#         # que no se pudo establecer la conexion
#         except (HTTPException, httpx.ConnectError, httpx.UnsupportedProtocol):

#             return JSONResponse(
#                 status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
#                 content={
#                     "estado": "error",
#                     "mensaje": "No se pudo establecer conexion con el servidor",
#                     "version": app.version,
#                     "timestamp": "2024-06-01T12:00:00Z",
#                 },
#             )


@app.get("/ping")
async def connection() -> JSONResponse:
    timestamp_actual = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")

    return JSONResponse(
        status_code=status.HTTP_200_OK,
        content={
            "estado": "ok",
            "mensaje": "Servidor de Cuentas Claras activo",
            "version": app.version,
            "timestamp": timestamp_actual,
        },
    )
