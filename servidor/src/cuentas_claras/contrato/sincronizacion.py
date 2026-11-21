from datetime import date
from decimal import Decimal
from typing import List

from pydantic import BaseModel, Field


class EventoCambioDTO(BaseModel):
    """Evento de cambio para sincronización (GET /cambios)."""

    id_evento: str = Field(
        ..., description="Identificador único e inmutable del evento"
    )
    tipo: str = Field(..., description="Tipo de evento (ej. GastoDeGrupoRegistrado)")
    grupo_id: str
    gasto_id: str
    concepto: str
    fecha: date
    moneda: str
    monto_parte: Decimal = Field(
        ..., decimal_places=2, description="Monto exacto de la parte"
    )
    incluye_redondeo: bool = Field(
        ..., description="True si incluye céntimos de residuo"
    )
    posicion_cursor: int = Field(..., description="Posición creciente en el registro")


class RespuestaCambiosDTO(BaseModel):
    """Respuesta del endpoint GET /cambios."""

    eventos: List[EventoCambioDTO] = Field(
        ..., max_length=100, description="Lote de eventos"
    )
    nuevo_cursor: int = Field(..., description="Cursor para la próxima petición")
    hay_mas: bool = Field(..., description="True si quedan más eventos por sincronizar")
