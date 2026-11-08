from collections.abc import Iterator
from datetime import datetime, timedelta, timezone
from pathlib import Path
from tempfile import TemporaryDirectory

from fastapi import FastAPI
from fastapi.exceptions import RequestValidationError
from sqlalchemy import create_engine, select
from sqlalchemy.orm import Session, sessionmaker

from cuentas_claras.app.main import manejar_datos_invalidos
from cuentas_claras.app.rutas.autenticacion import (
    obtener_db,
    obtener_reloj_auth,
    router,
)
from cuentas_claras.infraestructura.base_datos import Base
from cuentas_claras.infraestructura.modelos_usuario import UsuarioDB
from qa.entrega5.soporte_auth import EscenarioAuth


class RelojPrueba:
    def __init__(self) -> None:
        self._ahora = datetime.now(timezone.utc)

    def ahora(self) -> datetime:
        return self._ahora

    def avanzar(self, duracion: timedelta) -> None:
        self._ahora += duracion


def crear_escenario() -> EscenarioAuth:
    directorio = TemporaryDirectory()
    ruta = Path(directorio.name) / "usuarios.sqlite"
    engine = create_engine(
        f"sqlite:///{ruta.as_posix()}",
        connect_args={"check_same_thread": False},
    )
    Base.metadata.create_all(bind=engine)
    fabrica_sesiones = sessionmaker(
        bind=engine, autoflush=False, expire_on_commit=False
    )
    reloj = RelojPrueba()
    app = FastAPI()
    app.include_router(router)
    app.add_exception_handler(RequestValidationError, manejar_datos_invalidos)

    def obtener_sesion() -> Iterator[Session]:
        sesion = fabrica_sesiones()
        try:
            yield sesion
        finally:
            sesion.close()

    app.dependency_overrides[obtener_db] = obtener_sesion
    app.dependency_overrides[obtener_reloj_auth] = lambda: reloj

    def registro_persistido(correo: str) -> dict[str, object]:
        with Session(engine) as sesion:
            fila = sesion.scalar(
                select(UsuarioDB).where(UsuarioDB.correo == correo.casefold())
            )
            if fila is None:
                raise LookupError("No existe el usuario de prueba.")
            return {
                columna.name: getattr(fila, columna.name)
                for columna in UsuarioDB.__table__.columns
            }

    def cerrar() -> None:
        app.dependency_overrides.clear()
        engine.dispose()
        directorio.cleanup()

    return EscenarioAuth(
        app=app,
        ahora=reloj.ahora,
        avanzar_reloj=reloj.avanzar,
        registro_persistido=registro_persistido,
        cerrar=cerrar,
    )
