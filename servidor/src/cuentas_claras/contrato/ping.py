from pydantic import BaseModel


class PingRespuesta(BaseModel):
    estado: str
    mensaje: str
    version: str
    timestamp: str
