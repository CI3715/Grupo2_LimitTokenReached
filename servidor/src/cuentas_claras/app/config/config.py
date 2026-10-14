from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict

# Nota: este es el archivo de configuracion de las variables de entorno del sistema
# si agregan una variable al .env, deben colocarla aqui para que el sistema la reconozca

BASE_DIR = Path(__file__).resolve().parent.parent.parent.parent.parent.parent
print(f"BASE_DIR: {BASE_DIR}")


class Settings(BaseSettings):
    SERVER_HOST_IP: str = ""
    model_config = SettingsConfigDict(
        env_file=BASE_DIR / ".env", env_file_encoding="utf-8"
    )
