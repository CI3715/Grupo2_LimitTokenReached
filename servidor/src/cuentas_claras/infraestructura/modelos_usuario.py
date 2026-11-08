from datetime import datetime

from sqlalchemy import DateTime, String, UniqueConstraint
from sqlalchemy.orm import Mapped, mapped_column

from cuentas_claras.infraestructura.base_datos import Base


class UsuarioDB(Base):
    __tablename__ = "usuarios"
    __table_args__ = (
        UniqueConstraint("nombre_usuario", name="uq_usuarios_nombre"),
        UniqueConstraint("correo", name="uq_usuarios_correo"),
    )

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    nombre_usuario: Mapped[str] = mapped_column(String(30), nullable=False)
    nombre_visible: Mapped[str] = mapped_column(String(120), nullable=False)
    correo: Mapped[str] = mapped_column(String(320), nullable=False)
    hash_contrasena: Mapped[str] = mapped_column(String(255), nullable=False)
    fecha_alta: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), nullable=False
    )
