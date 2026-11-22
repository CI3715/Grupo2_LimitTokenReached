from datetime import date, datetime

from sqlalchemy import (
    Boolean,
    Date,
    DateTime,
    ForeignKey,
    Integer,
    String,
    UniqueConstraint,
)
from sqlalchemy.orm import Mapped, mapped_column

from cuentas_claras.infraestructura.base_datos import Base


class AmistadDB(Base):
    __tablename__ = "amistades"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    emisor_id: Mapped[str] = mapped_column(
        ForeignKey("usuarios.id"), nullable=False, index=True
    )
    destinatario_id: Mapped[str] = mapped_column(
        ForeignKey("usuarios.id"), nullable=False, index=True
    )
    estado: Mapped[str] = mapped_column(String(16), nullable=False, index=True)
    creada_en: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)


class GrupoDB(Base):
    __tablename__ = "grupos"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    nombre: Mapped[str] = mapped_column(String(120), nullable=False)
    descripcion: Mapped[str | None] = mapped_column(String(1000), nullable=True)
    moneda: Mapped[str] = mapped_column(String(3), nullable=False)
    actividad: Mapped[str | None] = mapped_column(String(200), nullable=True)
    creador_id: Mapped[str] = mapped_column(ForeignKey("usuarios.id"), nullable=False)
    creado_en: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)


class MiembroGrupoDB(Base):
    __tablename__ = "miembros_grupo"
    __table_args__ = (
        UniqueConstraint("grupo_id", "usuario_id", name="uq_miembro_grupo_usuario"),
    )

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    grupo_id: Mapped[str] = mapped_column(
        ForeignKey("grupos.id"), nullable=False, index=True
    )
    usuario_id: Mapped[str] = mapped_column(
        ForeignKey("usuarios.id"), nullable=False, index=True
    )
    rol: Mapped[str] = mapped_column(String(16), nullable=False)
    estado: Mapped[str] = mapped_column(String(16), nullable=False, index=True)
    invitado_por_id: Mapped[str] = mapped_column(
        ForeignKey("usuarios.id"), nullable=False
    )
    creada_en: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)


class GastoDB(Base):
    __tablename__ = "gastos"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    grupo_id: Mapped[str] = mapped_column(
        ForeignKey("grupos.id"), nullable=False, index=True
    )
    concepto: Mapped[str] = mapped_column(String(200), nullable=False)
    monto_centimos: Mapped[int] = mapped_column(Integer, nullable=False)
    fecha: Mapped[date] = mapped_column(Date, nullable=False)
    pagador_id: Mapped[str] = mapped_column(ForeignKey("usuarios.id"), nullable=False)
    regla: Mapped[str] = mapped_column(String(16), nullable=False)
    creado_en: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)


class ParteGastoDB(Base):
    __tablename__ = "partes_gasto"
    __table_args__ = (
        UniqueConstraint("gasto_id", "usuario_id", name="uq_parte_gasto_usuario"),
    )

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    gasto_id: Mapped[str] = mapped_column(
        ForeignKey("gastos.id"), nullable=False, index=True
    )
    usuario_id: Mapped[str] = mapped_column(
        ForeignKey("usuarios.id"), nullable=False, index=True
    )
    monto_centimos: Mapped[int] = mapped_column(Integer, nullable=False)
    recibio_residuo: Mapped[bool] = mapped_column(Boolean, nullable=False)

class CursorCambiosDB(Base):
    __tablename__ = "cursores_cambios"
    __table_args__ = (
        UniqueConstraint("usuario_id", name="uq_cursor_cambios_usuario"),
    )

    usuario_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("usuarios.id"),
        primary_key=True,
    )
    posicion: Mapped[int] = mapped_column(Integer, nullable=False, default=0)


class EventoCambioDB(Base):
    __tablename__ = "eventos_cambio"
    __table_args__ = (
        UniqueConstraint(
            "usuario_id",
            "posicion",
            name="uq_evento_cambio_usuario_posicion",
        ),
    )

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    usuario_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("usuarios.id"),
        nullable=False,
        index=True,
    )
    posicion: Mapped[int] = mapped_column(Integer, nullable=False)
    grupo_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("grupos.id"),
        nullable=False,
        index=True,
    )
    gasto_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("gastos.id"),
        nullable=False,
        index=True,
    )
    tipo: Mapped[str] = mapped_column(
        String(64),
        nullable=False,
        default="GastoDeGrupoRegistrado",
    )
    concepto: Mapped[str] = mapped_column(String(200), nullable=False)
    fecha: Mapped[date] = mapped_column(Date, nullable=False)
    moneda: Mapped[str] = mapped_column(String(3), nullable=False)
    monto_centimos: Mapped[int] = mapped_column(Integer, nullable=False)
    incluye_redondeo: Mapped[bool] = mapped_column(Boolean, nullable=False)
    creado_en: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        nullable=False,
    )


class IdempotenciaGastoDB(Base):
    __tablename__ = "idempotencia_gastos"
    __table_args__ = (
        UniqueConstraint(
            "clave",
            name="uq_idempotencia_gasto_clave",
        ),
    )

    clave: Mapped[str] = mapped_column(String(36), primary_key=True)
    usuario_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("usuarios.id"),
        nullable=False,
        index=True,
    )
    grupo_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("grupos.id"),
        nullable=False,
    )
    gasto_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("gastos.id"),
        nullable=False,
    )
    huella_datos: Mapped[str] = mapped_column(
        String(64),
        nullable=False,
    )
    creada_en: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        nullable=False,
    )