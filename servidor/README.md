# Servidor de Cuentas Claras

API REST con FastAPI para conectividad y autenticación. El servidor almacena
cuentas de autenticación, no información financiera personal.

## Requisitos

- Python 3.11 o más nuevo (lo probamos con 3.14)
- Git

No hace falta Docker para desarrollar.

## Instalación

Desde la carpeta `servidor`:

    python -m venv .venv
    .venv\Scripts\Activate.ps1
    pip install -e ".[qa-auth]"

En Linux o Mac, activa el entorno con `source .venv/bin/activate`.

Si PowerShell bloquea la activación, puedes ejecutar los comandos con
`.venv\Scripts\python.exe` sin activar el entorno.

## Variables de entorno

La configuración se lee desde el archivo `.env` de la raíz del repositorio.
Cópialo desde `.env.example` y genera un secreto para `JWT_SECRET` con:

    python -c "import secrets; print(secrets.token_hex(32))"

Configura `DATABASE_PATH` con una ruta relativa a la raíz del repositorio para
la SQLite **del servidor**. `JWT_SECRET` debe ser privado y nunca se sube a Git.
`JWT_ISSUER` identifica al emisor del token. `SERVER_HOST_IP` configura la URL
para el cliente Tauri y debe estar en el entorno al compilarlo o ejecutarlo;
Tauri no lee automáticamente el `.env` del servidor.
`JWT_TTL_MINUTES` vale 60 por defecto; se puede bajar temporalmente a `1` para
demostrar el vencimiento y luego restaurarlo a `60`.

## Cómo correrlo

Desde `servidor`, con el entorno activado:

    python -m uvicorn cuentas_claras.app.main:app --reload --app-dir src

Abre http://127.0.0.1:8000/docs para la documentación interactiva. Para
detenerlo, usa Ctrl+C.
HTTP es apropiado para desarrollo local; en un despliegue, configura HTTPS en
el proxy o servidor frontal.

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

Para correr la suite de aceptación E5 contra la API y una SQLite de prueba,
instala el extra y usa la fábrica incluida:

    python -m pip install -e ".[qa-auth]"

En Linux o Mac:

    JWT_TTL_MINUTES=60 CUENTAS_CLARAS_QA_AUTH_FACTORY=qa.entrega5.escenario_real:crear_escenario PYTHONPATH=src python -m pytest qa/entrega5

En PowerShell:

    $env:JWT_TTL_MINUTES = "60"
    $env:CUENTAS_CLARAS_QA_AUTH_FACTORY = "qa.entrega5.escenario_real:crear_escenario"
    $env:PYTHONPATH = "src"
    python -m pytest qa/entrega5

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
      requirements.txt

Las dependencias van hacia adentro: app usa a infraestructura, esta a aplicacion, y aplicacion a dominio. Nunca al revés. El contrato "layers" del pyproject.toml lo verifica con lint-imports.

La base SQLite de esta API es independiente de `libro.db`, que permanece en el
dispositivo. Ninguna ruta de autenticación recibe datos financieros.

## Esquema SQLite y migraciones

Al iniciar, la aplicación crea las tablas que falten con SQLAlchemy. La
migración `migrations/002_entrega6_social.sql` documenta y permite aplicar
explícitamente las tablas de amistades, grupos y gastos compartidos de E6. La
tabla `usuarios` debe existir previamente (la crea el esquema de autenticación
de E5).

Para aplicarla con la biblioteca SQLite de Python, desde la raíz del
repositorio usa la ruta configurada en `DATABASE_PATH` (por defecto,
`servidor/data/usuarios_auth.db`):

    python -c "import sqlite3; from pathlib import Path; db=sqlite3.connect('servidor/data/usuarios_auth.db'); db.executescript(Path('servidor/migrations/002_entrega6_social.sql').read_text()); db.close()"

Si `DATABASE_PATH` apunta a otro archivo, reemplaza la ruta de la base de datos
en el comando. La migración no copia usuarios ni gastos entre equipos; crea
solo el esquema y sus índices.

## OpenAPI

Desde `servidor`, con el entorno activado:

    PYTHONPATH=src python scripts/exportar_openapi.py

Comprueba que el contrato versionado esté actualizado con:

    PYTHONPATH=src python scripts/exportar_openapi.py --check