from dataclasses import dataclass
from datetime import datetime


@dataclass(frozen=True)
class Usuario:
    id: str
    nombre_usuario: str
    nombre_visible: str
    correo: str
    hash_contrasena: str
    fecha_alta: datetime
