from datetime import date
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, StrictStr


class _ContratoSocial(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=False)


class CrearAmistadRequest(_ContratoSocial):
    usuario: StrictStr


class RespuestaRequest(_ContratoSocial):
    aceptar: bool


class CrearGrupoRequest(_ContratoSocial):
    nombre: StrictStr = Field(min_length=1, max_length=120)
    descripcion: StrictStr | None = Field(default=None, max_length=1000)
    moneda: StrictStr = Field(pattern=r"^[A-Z]{3}$")
    actividad: StrictStr | None = Field(default=None, max_length=200)


class InvitarMiembroRequest(_ContratoSocial):
    usuario: StrictStr


class RegistrarGastoRequest(_ContratoSocial):
    concepto: StrictStr = Field(min_length=1, max_length=200)
    monto: StrictStr
    fecha: date
    pagador: StrictStr
    regla: Literal["iguales", "porcentajes", "fijos"]
    participantes: list[StrictStr] = Field(min_length=1)
    valores: dict[StrictStr, StrictStr] = Field(default_factory=dict)
