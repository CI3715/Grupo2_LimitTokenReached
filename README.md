# Cuentas Claras

Aplicación de gestión de finanzas personales con una interfaz web y una base multiplataforma construida con Tauri. Proyecto del **Grupo 2 — Sección 1**, desarrollado para el curso **CI3715**, período septiembre–diciembre de 2026.

## Descripción

Cuentas Claras busca facilitar el registro de cuentas y movimientos financieros, la conversión de monedas y la importación y exportación del libro personal entre dispositivos. Su diseño contempla una aplicación de escritorio y Android, junto con un servidor para coordinar información compartida entre usuarios.

### Principios de manejo de datos

- **Aplicación local:** almacenar los datos financieros y personales en el dispositivo del usuario. Su exposición requiere autorización de la persona.
- **Servidor:** gestionar la información necesaria para las funciones compartidas, como amistades, solicitudes y grupos.

Estos principios describen el diseño previsto del producto; las funcionalidades se incorporan de forma incremental.

## Estado actual

| Área | Implementación disponible |
| --- | --- |
| Interfaz | Inicio en Mis cuentas y sección Conexión en `/conexion/`. |
| Frontend | Next.js, React, TypeScript y Tailwind CSS; exportación estática para Tauri. |
| Aplicación nativa | Estructura y configuración inicial de Tauri. |
| Backend | Servidor FastAPI con endpoint `GET /ping`. |
| Comunicación de la interfaz | Conexión mediante `ping_servidor` de Tauri; cuentas con datos de muestra y estado temporal de React. |
| Gestión financiera | Vista de cuentas, subtotales por moneda y formulario; integración local pendiente. |

Mis cuentas es una vista previa sin persistencia ni llamadas al backend. En `/conexion/`, la comprobación real requiere ejecutar la aplicación con Tauri y tener el servidor disponible; `npm run dev` permite revisar la interfaz en el navegador.

## Tecnologías y arquitectura

| Componente | Tecnologías | Responsabilidad |
| --- | --- | --- |
| Interfaz | Next.js · React · TypeScript · Tailwind CSS | Presentación e interacción con el usuario. |
| Contenedor nativo | Tauri · Rust | Ejecución de la interfaz como aplicación nativa. |
| Servidor | FastAPI · Python · Uvicorn | API y coordinación de información compartida. |

El backend se organiza siguiendo una arquitectura hexagonal: dominio, aplicación, infraestructura, contratos y arranque de la API. El proyecto incluye configuración de Import Linter para revisar las dependencias entre capas y reglas de ESLint para organizar las dependencias del frontend.

```text
Grupo2_LimitTokenReached/
├── app/
│   ├── src/app/                 # Páginas, estilos y layout del frontend
│   ├── src-tauri/               # Proyecto nativo y configuración de Tauri
│   ├── public/                  # Recursos estáticos
│   └── package.json             # Dependencias y scripts del frontend
├── servidor/
│   ├── src/cuentas_claras/
│   │   ├── dominio/             # Reglas del negocio
│   │   ├── aplicacion/          # Casos de uso y puertos
│   │   ├── infraestructura/     # Adaptadores e implementaciones
│   │   ├── contrato/            # Contratos de la API
│   │   └── app/                 # Configuración de FastAPI
│   ├── src/main.py             # Arranque de FastAPI
│   ├── tests/                   # Pruebas del servidor
│   ├── pyproject.toml
│   └── requirements.txt
├── docs/adr/                    # Decisiones de arquitectura
└── LICENSE
```

## Desarrollo local

### Requisitos

- Git.
- Node.js 20.9 o superior y npm. Para Tauri también se necesita pnpm, utilizado por sus comandos de arranque y compilación.
- Python 3.11 o superior.
- Para ejecutar Tauri: Rust, Cargo y las dependencias nativas correspondientes al sistema operativo.

### Obtener el proyecto

```bash
git clone https://github.com/CI3715/Grupo2_LimitTokenReached.git
cd Grupo2_LimitTokenReached
```

### Ejecutar el frontend

Desde la raíz del repositorio:

```bash
cd app
npm install
npm run dev
```

La interfaz estará disponible en **http://localhost:3000**. Los archivos de la interfaz se encuentran en `app/src/app/`.

En PowerShell, si la política de ejecución bloquea `npm.ps1`, utiliza `npm.cmd` en lugar de `npm`.

### Ejecutar el backend

Los siguientes comandos se ejecutan desde la raíz del repositorio. No es necesario activar el entorno virtual.

**Windows — PowerShell:**

```powershell
python -m venv servidor/.venv
./servidor/.venv/Scripts/python.exe -m pip install -e ./servidor
./servidor/.venv/Scripts/python.exe -m pip install -r ./servidor/requirements.txt pydantic-settings
./servidor/.venv/Scripts/python.exe -m uvicorn main:app --app-dir servidor/src --reload
```

**Linux / macOS:**

```bash
python3 -m venv servidor/.venv
./servidor/.venv/bin/python -m pip install -e ./servidor
./servidor/.venv/bin/python -m pip install -r ./servidor/requirements.txt pydantic-settings
./servidor/.venv/bin/python -m uvicorn main:app --app-dir servidor/src --reload
```

El punto de entrada del servidor es `servidor/src/main.py`. `--app-dir servidor/src` permite resolver los imports de sus capas desde la raíz del repositorio.

El endpoint actual no utiliza `SERVER_HOST_IP` para consultar otro servicio; no es necesario definir esa variable para iniciar el backend.

Con el servidor en ejecución:

| Recurso | Dirección |
| --- | --- |
| Documentación interactiva | http://127.0.0.1:8000/docs |
| Esquema OpenAPI | http://127.0.0.1:8000/openapi.json |
| Endpoint de conexión | http://127.0.0.1:8000/ping |

`GET /ping` responde con HTTP **200 OK** y un objeto JSON de estado, mensaje, versión y timestamp. En el flujo de esta entrega, quien realiza el GET es Rust desde la aplicación; `/ping` solo responde el contrato JSON. La latencia se mide en la aplicación cliente y no forma parte del contrato del endpoint.

### Contrato y flujo requerido para la Entrega 1

La entrega requiere el siguiente contrato de respuesta para `GET /ping` (el timestamp mostrado es un ejemplo):

```json
{
  "estado": "ok",
  "mensaje": "Servidor Cuentas Claras activo",
  "version": "0.1.0",
  "timestamp": "2026-10-03T10:00:00Z"
}
```

El flujo que debe completar la aplicación de escritorio con Tauri 2 y Next.js es:

1. La interfaz muestra el botón **«Verificar Conexión con el Servidor»** y un indicador de conexión.
2. Al presionarlo, Next.js llama a `invoke('ping_servidor')` mediante su módulo de comunicación.
3. El comando Rust realiza `GET http://localhost:8000/ping` mediante el adaptador HTTP y mide el tiempo de respuesta en milisegundos.
4. FastAPI devuelve el contrato JSON y la interfaz muestra éxito o error, el mensaje y la versión recibidos, y la latencia.

La UI se encarga de presentar datos y capturar eventos; el módulo de comunicación realiza la invocación a Tauri; Rust gestiona la petición HTTP; y el backend recibe la petición y responde JSON.

La regla arquitectónica 2.3 de la guía, relativa a dependencias externas, exige manejar la red y el reloj mediante puertos y adaptadores; no prohíbe por sí misma que `/ping` consulte otra URL. Esta regla es distinta del apartado 2.3 de la especificación de la Entrega 1, que define el contrato de `GET /ping`. Para el flujo de esta entrega, el backend solo responde el contrato. Conforme a la arquitectura descrita en [ADR-004](docs/adr/004-arquitectura-hexagonal.md), el reloj debe exponerse mediante un puerto de aplicación y obtener la hora del sistema en un adaptador de infraestructura, en lugar de llamar directamente a `datetime.now()` desde la ruta.

La sección Conexión conserva el flujo de `main`: `usePing` → `pingService` → `ping_servidor` → `GET /ping`. Este flujo es independiente de Mis cuentas y solo se ejecuta al pulsar el botón de comprobación.

### Ejecutar la aplicación de escritorio

Desde `app/`, con las dependencias nativas de Tauri instaladas:

```bash
npm run tauri dev
```

Tauri inicia el servidor de desarrollo del frontend mediante su configuración. Para generar el paquete nativo:

```bash
npm run tauri build
```

El comando Rust utiliza por defecto `http://localhost:8000` como URL base y añade `/ping`. Si se necesita otra dirección, `SERVER_HOST_IP` debe contener la URL base completa, sin `/docs` ni `/ping`, y estar definida en el entorno al compilar o ejecutar `npm run tauri dev`: el código Rust la lee en compilación mediante `option_env!`. Un archivo `.env` del backend no configura automáticamente esta variable para Tauri.

## Verificaciones

Desde `app/`:

```bash
npm run lint
npm run build
```

La compilación revisa los tipos de TypeScript y genera la exportación estática en `app/out/`.

El servidor incluye herramientas de pruebas y revisión: Pytest, Ruff, Black, Mypy e Import Linter. Su configuración se encuentra en `servidor/pyproject.toml`; la existencia de estas herramientas no implica que todas las verificaciones pasen en el estado actual.

Desde `servidor/`, con el entorno virtual activado y las herramientas instaladas:

```bash
pytest
ruff check .
black --check .
mypy
```

Pytest ejecuta las pruebas; Ruff revisa el código con las reglas configuradas; Black comprueba el formato; y Mypy verifica los tipos estáticos. Para comprobar las dependencias entre capas con Import Linter:

```bash
# Linux / macOS
PYTHONPATH=src lint-imports
```

```powershell
# Windows — PowerShell
$env:PYTHONPATH = "src"
lint-imports
```

## Contribuciones

1. Crear una rama a partir de `main` para cada cambio.
2. Mantener el alcance del cambio claro y actualizar la documentación cuando corresponda.
3. Ejecutar las verificaciones relacionadas con los archivos modificados.
4. Abrir un pull request hacia `main`, describiendo el cambio y las comprobaciones realizadas.

No incluir archivos `.env`, credenciales ni artefactos de compilación en los commits.

## Licencia

Este proyecto se distribuye bajo la **licencia Apache 2.0**. Consulta el archivo [LICENSE](LICENSE) para conocer sus términos.

## Vista Mis cuentas — Entrega 2

La aplicación abre en **Mis cuentas**, con nombre, tipo, moneda y saldo, subtotales separados por moneda y formulario **Nueva cuenta**. Incluye ejemplos de efectivo VES, banco USD y billetera USDT, además del catálogo VES, USD, EUR y USDT.

Esta rama cubre la presentación con datos de muestra y estado temporal de React. Recargar restablece los ejemplos. La persistencia SQLite, los comandos `listar_cuentas`, `listar_monedas` y `crear_cuenta`, los eventos y las validaciones de dominio quedan a cargo de la integración posterior. Las cuentas no realizan peticiones HTTP ni invocaciones Tauri.

Consulta el [contrato de integración y las comprobaciones de la vista](docs/qa-mis-cuentas.md).
