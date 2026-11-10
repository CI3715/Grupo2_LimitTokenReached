"""App real con BD aislada y reloj controlado; sin sustituir autorización ni rutas."""

from collections.abc import AsyncIterator, Iterator
from contextlib import asynccontextmanager, contextmanager
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from functools import lru_cache
from uuid import NAMESPACE_DNS, uuid5

from fastapi import FastAPI
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker
from sqlalchemy.pool import StaticPool

from cuentas_claras.app.main import app
from cuentas_claras.app.rutas.autenticacion import obtener_reloj_auth
from cuentas_claras.infraestructura.base_datos import Base, obtener_db
from cuentas_claras.infraestructura.modelos_usuario import UsuarioDB
from cuentas_claras.infraestructura.seguridad_auth import crear_hash, crear_token

CLAVE_PRUEBA = "Clave-exclusiva-QA-123"
USUARIOS = ("ana", "beto", "carla", "diana")
USUARIO_IDS = {
    usuario: str(uuid5(NAMESPACE_DNS, f"qa.e6.{usuario}")) for usuario in USUARIOS
}


class RelojPrueba:
    def __init__(self) -> None:
        self.instante = datetime.now(timezone.utc).replace(microsecond=0)

    def ahora(self) -> datetime:
        return self.instante

    def avanzar(self, segundos: int) -> None:
        self.instante += timedelta(seconds=segundos)


@dataclass
class EscenarioSocial:
    cliente: TestClient
    reloj: RelojPrueba
    fabrica_db: sessionmaker[Session]
    tokens: dict[str, str] = field(repr=False)

    def cabecera(self, usuario: str) -> dict[str, str]:
        return {"Authorization": f"Bearer {self.tokens[usuario]}"}


@lru_cache(maxsize=1)
def _hash_prueba() -> str:
    return crear_hash(CLAVE_PRUEBA)


@asynccontextmanager
async def _ciclo_prueba(_: FastAPI) -> AsyncIterator[None]:
    # Las tablas ya se crean en la BD aislada, nunca en el engine global.
    yield


@contextmanager
def crear_escenario() -> Iterator[EscenarioSocial]:
    engine = create_engine(
        "sqlite://", connect_args={"check_same_thread": False}, poolclass=StaticPool
    )
    Base.metadata.create_all(engine)
    fabrica = sessionmaker(bind=engine, expire_on_commit=False)
    reloj = RelojPrueba()
    with fabrica() as db:
        db.add_all(
            UsuarioDB(
                id=USUARIO_IDS[usuario],
                nombre_usuario=usuario,
                nombre_visible=usuario.title(),
                correo=f"{usuario}@example.com",
                hash_contrasena=_hash_prueba(),
                fecha_alta=reloj.ahora(),
            )
            for usuario in USUARIOS
        )
        db.commit()

    def db_prueba() -> Iterator[Session]:
        with fabrica() as db:
            yield db

    anteriores = app.dependency_overrides.copy()
    ciclo_anterior = app.router.lifespan_context
    app.dependency_overrides[obtener_db] = db_prueba
    app.dependency_overrides[obtener_reloj_auth] = lambda: reloj
    app.router.lifespan_context = _ciclo_prueba
    try:
        tokens = {
            usuario: crear_token(USUARIO_IDS[usuario], reloj.ahora())[0]
            for usuario in USUARIOS
        }
        with TestClient(app) as cliente:
            yield EscenarioSocial(cliente, reloj, fabrica, tokens)
    finally:
        app.dependency_overrides.clear()
        app.dependency_overrides.update(anteriores)
        app.router.lifespan_context = ciclo_anterior
        engine.dispose()
