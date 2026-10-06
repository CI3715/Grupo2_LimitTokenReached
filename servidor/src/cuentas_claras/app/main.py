#Archivo principal de la aplicacion

from fastapi import FastAPI

app = FastAPI(
    title="Cuentas Claras - Servidor",
    version="0.1.0",
)

@app.get("/")
def conection() -> dict:
    return {"message": "Conexión exitosa", "rescode": 200}

