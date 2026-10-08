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
| Interfaz | Pantalla de estado de conexión con modos claro y oscuro. |
| Frontend | Next.js, React, TypeScript y Tailwind CSS; exportación estática para Tauri. |
| Aplicación nativa | Estructura y configuración inicial de Tauri. |
| Backend | Servidor FastAPI con endpoint `GET /ping`. |
| Comunicación de la interfaz | Verificación simulada; pendiente de conectar con el backend. |
| Gestión financiera | Funcionalidad prevista para próximas entregas. |

**El indicador «Conectado» de la interfaz no verifica todavía la disponibilidad real del servidor.**

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
│   │   └── app/                 # Configuración y arranque de FastAPI
│   ├── tests/                   # Pruebas del servidor
│   ├── pyproject.toml
│   └── requirements.txt
├── docs/adr/                    # Decisiones de arquitectura
└── LICENSE
```

## Desarrollo local

### Requisitos

- Git.
- Node.js y npm. El frontend se ha compilado localmente con Node.js 22.
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
npm ci
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
$env:SERVER_HOST_IP = "http://127.0.0.1:8000/docs"
cd servidor/src/cuentas_claras/app
../../../.venv/Scripts/python.exe -m uvicorn main:app --reload
```

**Linux / macOS:**

```bash
python3 -m venv servidor/.venv
./servidor/.venv/bin/python -m pip install -e ./servidor
./servidor/.venv/bin/python -m pip install -r ./servidor/requirements.txt pydantic-settings
export SERVER_HOST_IP="http://127.0.0.1:8000/docs"
cd servidor/src/cuentas_claras/app
../../../.venv/bin/python -m uvicorn main:app --reload
```

El arranque se realiza desde esa carpeta porque la implementación actual importa `config.config` como un módulo local. Se instala `pydantic-settings` explícitamente porque el código lo utiliza y todavía no está declarado en las dependencias.

`SERVER_HOST_IP` debe contener una **URL completa**, incluido `http://` o `https://`. También puede definirse en un archivo `.env` en la raíz del repositorio. El ejemplo utiliza la documentación del servidor local como destino para comprobar el flujo de `/ping`; sustituye esa URL por la del servicio que quieras consultar.

Con el servidor en ejecución:

| Recurso | Dirección |
| --- | --- |
| Documentación interactiva | http://127.0.0.1:8000/docs |
| Esquema OpenAPI | http://127.0.0.1:8000/openapi.json |
| Endpoint de conexión | http://127.0.0.1:8000/ping |

`GET /ping` consulta la URL configurada y devuelve información de estado, versión, fecha y latencia. Ejecutar ambos servicios todavía no conecta automáticamente la interfaz con la API.

### Ejecutar la aplicación de escritorio

Desde `app/`, con las dependencias nativas de Tauri instaladas:

```bash
npm run tauri dev
```

Tauri inicia el servidor de desarrollo del frontend mediante su configuración. Para generar el paquete nativo:

```bash
npm run tauri build
```

## Verificaciones

Desde `app/`:

```bash
npm run lint
npm run build
```

La compilación revisa los tipos de TypeScript y genera la exportación estática en `app/out/`.

El servidor incluye herramientas de pruebas y revisión: Pytest, Ruff, Black, Mypy e Import Linter. Su configuración se encuentra en `servidor/pyproject.toml`; la existencia de estas herramientas no implica que todas las verificaciones pasen en el estado actual.

## Contribuciones

1. Crear una rama a partir de `main` para cada cambio.
2. Mantener el alcance del cambio claro y actualizar la documentación cuando corresponda.
3. Ejecutar las verificaciones relacionadas con los archivos modificados.
4. Abrir un pull request hacia `main`, describiendo el cambio y las comprobaciones realizadas.

No incluir archivos `.env`, credenciales ni artefactos de compilación en los commits.

## Licencia

Este proyecto se distribuye bajo la **licencia Apache 2.0**. Consulta el archivo [LICENSE](LICENSE) para conocer sus términos.
