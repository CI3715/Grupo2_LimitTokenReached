from collections.abc import Generator
from pathlib import Path

from sqlalchemy import create_engine
from sqlalchemy.orm import DeclarativeBase, Session, sessionmaker

from cuentas_claras.app.config.config import BASE_DIR, settings


class Base(DeclarativeBase):
    pass


ruta_db = Path(settings.DATABASE_PATH)
if not ruta_db.is_absolute():
    ruta_db = BASE_DIR / ruta_db
ruta_db.parent.mkdir(parents=True, exist_ok=True)

engine = create_engine(
    f"sqlite:///{ruta_db.as_posix()}",
    connect_args={"check_same_thread": False},
)
SessionLocal = sessionmaker(bind=engine, autoflush=False, expire_on_commit=False)


def obtener_db() -> Generator[Session, None, None]:
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
