from pathlib import Path

from pydantic import SecretStr
from pydantic_settings import BaseSettings, SettingsConfigDict

BASE_DIR = Path(__file__).resolve().parents[5]


class Settings(BaseSettings):
    SERVER_HOST_IP: str = "http://localhost:8000"
    DATABASE_PATH: str = "servidor/data/usuarios_auth.db"
    JWT_SECRET: SecretStr = SecretStr("")
    JWT_ISSUER: str = "cuentas-claras"
    JWT_TTL_MINUTES: int = 1

    model_config = SettingsConfigDict(
        env_file=BASE_DIR / ".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )


settings = Settings()
if len(settings.JWT_SECRET.get_secret_value().encode("utf-8")) < 32:
    raise RuntimeError("JWT_SECRET debe tener al menos 32 bytes.")
if settings.JWT_TTL_MINUTES < 1:
    raise RuntimeError("JWT_TTL_MINUTES debe ser al menos 1.")

__all__ = ["BASE_DIR", "Settings", "settings"]
