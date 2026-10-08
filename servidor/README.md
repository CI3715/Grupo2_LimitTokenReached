# Servidor de Cuentas Claras

Servidor REST hecho con FastAPI. Por ahora solo tiene el esqueleto del proyecto; el endpoint GET /ping se agrega en la siguiente rama.

## Requisitos

- Python 3.11 o más nuevo (lo probamos con 3.14)
- Git

No hace falta Docker para desarrollar.

## Instalación

Desde la carpeta servidor:

    python -m venv .venv
    .venv\Scripts\Activate.ps1
    pip install -e .
    pip install -r requirements-dev.txt

En Linux o Mac, el segundo comando es: source .venv/bin/activate

Si PowerShell no deja activar el entorno, corre una vez este comando y vuelve a intentar:

    Set-ExecutionPolicy -Scope CurrentUser RemoteSigned

Si el proyecto está dentro de una carpeta de OneDrive, la instalación puede tardar bastante porque sincroniza cada archivo que se crea. Pausar la sincronización ayuda.

## Cómo correrlo

Con el entorno activado (se ve (.venv) al inicio de la terminal):

    uvicorn cuentas_claras.app.main:app --reload

Después abre http://127.0.0.1:8000/docs para ver la documentación de Swagger. Para apagarlo, Ctrl+C.

## Pruebas y revisiones

    pytest
    ruff check .
    black --check .
    mypy
    lint-imports

- pytest corre las pruebas.
- ruff y black revisan el estilo del código. Si black se queja, "black ." lo arregla solo.
- mypy revisa los tipos en modo estricto.
- lint-imports revisa que se respete la regla de dependencia entre capas.

## Estructura

    servidor/
      src/cuentas_claras/
        dominio/          reglas del negocio, no importa nada de afuera
        aplicacion/       casos de uso y puertos
        infraestructura/  implementaciones reales de los puertos
        app/              FastAPI, rutas y arranque (main.py)
        contrato/         modelos de lo que viaja por la API
      tests/
      pyproject.toml
      requirements-dev.txt

Las dependencias van hacia adentro: app usa a infraestructura, esta a aplicacion, y aplicacion a dominio. Nunca al revés. El contrato "layers" del pyproject.toml lo verifica con lint-imports.

## Pendiente

- Endpoint GET /ping
- Exportar el openapi.yaml
- CI en GitHub Actions
- Configuración para desplegar en Render
