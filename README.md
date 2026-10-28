# Cuentas Claras

Aplicación de gestión de finanzas personales con una interfaz web y una base multiplataforma construida con Tauri. Proyecto del Grupo 2 — Sección 1, desarrollado para el curso CI3715, período septiembre–diciembre de 2026.

## Descripción

Cuentas Claras busca facilitar el registro de cuentas y movimientos financieros, la conversión de monedas y la importación y exportación del libro personal entre dispositivos. Su diseño contempla una aplicación de escritorio y Android, junto con un servidor para coordinar información compartida entre usuarios.

### Principios de manejo de datos

- **Aplicación local:** almacenar los datos financieros y personales en el dispositivo del usuario. Su exposición requiere autorización de la persona.
- **Servidor:** gestionar la información necesaria para las funciones compartidas, como amistades, solicitudes y grupos.

Estos principios describen el diseño previsto del producto; las funcionalidades se incorporan de forma incremental.

## Estado actual

| Área               | Implementación disponible                                                                                                                                                                                             |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Interfaz           | Pantalla de estado de conexión con modos claro y oscuro. Listado y creación de cuentas. Detalle de cuenta con saldo, composición y lista de movimientos. Formulario para registrar movimientos y flujo de corrección. |
| Frontend           | Next.js, React, TypeScript y Tailwind CSS; exportación estática para Tauri.                                                                                                                                           |
| Aplicación nativa  | Estructura y configuración inicial de Tauri. Gestión de almacenamiento local.                                                                                                                                         |
| Backend            | Servidor FastAPI con endpoint `GET /ping`.                                                                                                                                                                            |
| Gestión financiera | **Entrega 3 completada:** Registro de gastos e ingresos, corrección de movimientos con ajustes inmutables, cálculo automático de saldo y funcionamiento 100% offline (Local-First).                                   |

El indicador «Conectado» de la interfaz no verifica todavía la disponibilidad real del servidor en todas las rutas, pero el núcleo de la aplicación funciona de forma autónoma.

### Entrega 4: monedas, tasas y patrimonio

Están integrados los DTOs de monedas, tasas y patrimonio y las pruebas de contrato de tasas y patrimonio. Estas pruebas construyen requests/responses; todavía no verifican rechazos reales del dominio, selección de tasas, inversas ni redondeo. En el código actual faltan los casos de uso, la persistencia de eventos y los comandos Tauri de la Entrega 4, junto con sus pantallas.

Desde la raíz del repositorio:

```bash
cd app
# Suite de contratos y núcleo (incluye las pruebas de contrato de Entrega 4)
cargo test -p contrato -p aplicacion -p dominio
# Ejecutar únicamente cada grupo nuevo de contratos
cargo test -p contrato tasa_tests::
cargo test -p contrato patrimonio_tests::
```

La [guía de QA de Entrega 4](docs/qa-entrega-4.md) detalla los pasos y resultados del ejemplo 2.3: **316.100,00 VES**, bloqueo por falta de **USDT→USD** y **350,00 USD** al registrar la tasa faltante. Incluye selección por fecha, validaciones, autonomía y pendientes para la evaluación. El flujo manual podrá ejecutarse cuando se integren backend y frontend; no está certificado por las pruebas de DTOs.

Decisiones propuestas para revisión del equipo: [ADR-012: eventos de tasas y monedas](docs/adr/012-eventos-tasas-monedas.md) y [ADR-013: patrimonio y precisión decimal](docs/adr/013-patrimonio-multimoneda.md).

## Tecnologías y arquitectura

| Componente        | Tecnologías                                 | Responsabilidad                                  |
| ----------------- | ------------------------------------------- | ------------------------------------------------ |
| Interfaz          | Next.js · React · TypeScript · Tailwind CSS | Presentación e interacción con el usuario.       |
| Contenedor nativo | Tauri · Rust                                | Ejecución de la interfaz como aplicación nativa. |
| Servidor          | FastAPI · Python · Uvicorn                  | API y coordinación de información compartida.    |

El backend se organiza siguiendo una arquitectura hexagonal: dominio, aplicación, infraestructura, contratos y arranque de la API. El proyecto incluye configuración de Import Linter para revisar las dependencias entre capas y reglas de ESLint para organizar las dependencias del frontend.

```text
Grupo2_LimitTokenReached/
├── app/
│   ├── src/app/                 # Páginas, estilos y layout del frontend
│   ├── src-tauri/               # Proyecto nativo y configuración de Tauri
│   ├── crates/                  # Crates de Rust (dominio, aplicacion, infraestructura, contrato)
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

---

## Desarrollo local

### Requisitos

- Git.
- Node.js y pnpm. El frontend se ha compilado localmente con Node.js 22.
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
pnpm install --frozen-lockfile
pnpm run dev
```

La interfaz estará disponible en `http://localhost:3000`. Los archivos de la interfaz se encuentran en `app/src/app/`.

> **Nota:** En PowerShell, si la política de ejecución bloquea `pnpm.ps1`, utiliza `pnpm.cmd` en lugar de `pnpm`.

### Ejecutar el backend

Los siguientes comandos se ejecutan desde la raíz del repositorio. No es necesario activar el entorno virtual.

#### Windows — PowerShell:

```powershell
python -m venv servidor/.venv
./servidor/.venv/Scripts/python.exe -m pip install -e ./servidor
./servidor/.venv/Scripts/python.exe -m pip install -r ./servidor/requirements.txt pydantic-settings
cd servidor/src/cuentas_claras/app
../../../.venv/Scripts/python.exe -m uvicorn main:app --reload
```

#### Linux / macOS:

```bash
python3 -m venv servidor/.venv
./servidor/.venv/bin/python -m pip install -e ./servidor
./servidor/.venv/bin/python -m pip install -r ./servidor/requirements.txt pydantic-settings
cd servidor/src/cuentas_claras/app
../../../.venv/bin/python -m uvicorn main:app --reload
```

El arranque se realiza desde esa carpeta porque la implementación actual importa `config.config` como un módulo local. Se instala `pydantic-settings` explícitamente porque el código lo utiliza y todavía no está declarado en las dependencias.

El endpoint actual no utiliza `SERVER_HOST_IP` para consultar otro servicio; no es necesario definir esa variable para iniciar el backend.

Con el servidor en ejecución:

| Recurso                   | Dirección                            |
| ------------------------- | ------------------------------------ |
| Documentación interactiva | `http://127.0.0.1:8000/docs`         |
| Esquema OpenAPI           | `http://127.0.0.1:8000/openapi.json` |
| Endpoint de conexión      | `http://127.0.0.1:8000/ping`         |

`GET /ping` responde con HTTP 200 OK y un objeto JSON de estado, mensaje, versión y timestamp. En el flujo de esta entrega, quien realiza el GET es Rust desde la aplicación; `/ping` solo responde el contrato JSON. La latencia se mide en la aplicación cliente y no forma parte del contrato del endpoint.

---

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

1. La interfaz muestra el botón «Verificar Conexión con el Servidor» y un indicador de conexión.
2. Al presionarlo, Next.js llama a `invoke('ping_servidor')` mediante su módulo de comunicación.
3. El comando Rust realiza `GET http://localhost:8000/ping` mediante el adaptador HTTP y mide el tiempo de respuesta en milisegundos.
4. FastAPI devuelve el contrato JSON y la interfaz muestra éxito o error, el mensaje y la versión recibidos, y la latencia.

La UI se encarga de presentar datos y capturar eventos; el módulo de comunicación realiza la invocación a Tauri; Rust gestiona la petición HTTP; y el backend recibe la petición y responde JSON.

La regla arquitectónica 2.3 de la guía, relativa a dependencias externas, exige manejar la red y el reloj mediante puertos y adaptadores; no prohíbe por sí misma que `/ping` consulte otra URL. Esta regla es distinta del apartado 2.3 de la especificación de la Entrega 1, que define el contrato de `GET /ping`. Para el flujo de esta entrega, el backend solo responde el contrato. Conforme a la arquitectura descrita en ADR-004, el reloj debe exponerse mediante un puerto de aplicación y obtener la hora del sistema en un adaptador de infraestructura, en lugar de llamar directamente a `datetime.now()` desde la ruta.

> **Pendientes en el código actual:** conectar la interfaz, que todavía simula la verificación, con `invoke('ping_servidor')`; ajustar el texto del botón al solicitado; cambiar el mensaje del backend de "Servidor de Cuentas Claras activo" a "Servidor Cuentas Claras activo"; y sustituir la llamada directa a `datetime.now()` por el puerto de reloj y su adaptador. El comando Rust `ping_servidor` y su adaptador HTTP ya existen, pero aún no son invocados por la interfaz. Ejecutar ambos servicios todavía no completa la prueba de conexión de extremo a extremo.

---

### Ejecutar la aplicación de escritorio

Desde `app/`, con las dependencias nativas de Tauri instaladas:

```bash
pnpm run tauri dev
```

Tauri inicia el servidor de desarrollo del frontend mediante su configuración. Para generar el paquete nativo:

```bash
pnpm run tauri build
```

El comando Rust utiliza por defecto `http://localhost:8000` como URL base y añade `/ping`. Si se necesita otra dirección, `SERVER_HOST_IP` debe contener la URL base completa, sin `/docs` ni `/ping`, y estar definida en el entorno al compilar o ejecutar `pnpm run tauri dev`: el código Rust la lee en compilación mediante `option_env!`. Un archivo `.env` del backend no configura automáticamente esta variable para Tauri.

---

## Verificaciones

Desde `app/`:

```bash
pnpm run lint
pnpm run build
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

# Windows — PowerShell
$env:PYTHONPATH = "src"
lint-imports
```

### Núcleo Rust (Entrega 3)

El proyecto incluye pruebas unitarias para los invariantes y validaciones de dominio. Desde `app/`:

```bash
# Ejecutar pruebas de invariantes y validaciones de los casos de uso
cargo test -p contrato -p aplicacion -p dominio
```

---

## Creación de Cuentas y Almacenamiento Local (Entrega 2)

### Almacenamiento

Las cuentas se guardan localmente en el dispositivo usando SQLite embebido (`rusqlite`). El archivo de base de datos se almacena de forma segura en el directorio de datos nativo de la aplicación (`app_data_dir` de Tauri), garantizando que los datos persistan al cerrar y reabrir la app, y que no salgan del dispositivo (Local-First).

### Ejecución de Pruebas Automatizadas (Entrega 2)

El proyecto incluye pruebas unitarias para los invariantes de creación de cuentas (INV-01, INV-03, INV-18) y validaciones de `CrearCuenta`, ejecutadas con un repositorio en memoria (sin base de datos real ni red), cumpliendo con los criterios de aceptación de la Entrega 2.

---

## Registrar Movimientos y Ver Saldos (Entrega 3)

### Casos de Uso Implementados

La Entrega 3 extiende la aplicación para permitir el registro de gastos, ingresos y correcciones de movimientos, con cálculo automático del saldo. Los casos de uso implementados son:

- **`RegistrarMovimiento`:** Permite anotar un ingreso o gasto en una cuenta, validando monto, categoría, fecha y moneda.
- **`CorregirMovimiento`:** Permite corregir un movimiento existente registrando un ajuste inmutable que referencia al original (INV-04).
- **`CrearCategoria`:** Permite al usuario definir nuevas categorías de gasto o ingreso.

### Invariantes Validados

Además de los invariantes de la Entrega 2, la Entrega 3 valida:

- **INV-04 (Inmutabilidad de Movimientos):** Un movimiento registrado nunca se modifica ni se elimina. Las correcciones se registran como nuevos eventos de ajuste. La tabla de eventos solo admite `INSERT`, nunca `UPDATE` ni `DELETE`.
- **INV-05 (Cálculo de Saldo):** El saldo mostrado siempre coincide con `saldo inicial + ingresos − gastos ± ajustes`. Cualquier discrepancia es un defecto de severidad máxima (RF-LIB-09).

### Ejemplo de Corrección de Movimiento (Sección 2.2 de la Especificación)

El siguiente ejemplo reproduce el flujo de corrección exigido por la rúbrica. Cuenta "Billetera dólares" en USD:

| Paso | Movimiento                                  | Efecto   | Saldo    |
| ---- | ------------------------------------------- | -------- | -------- |
| 1    | Saldo inicial                               | —        | 1.000,00 |
| 2    | Ingreso · Salario                           | + 800,00 | 1.800,00 |
| 3    | Gasto · Alimentación (era 50, se anotó 500) | − 500,00 | 1.300,00 |
| 4    | Ajuste del paso 3 (monto correcto: 50,00)   | + 450,00 | 1.750,00 |

El paso 3 sigue en el historial marcado como corregido, y el saldo final es el correcto: $1.000 + 800 - 50 = 1.750$.

Este ejemplo se reproduce matemáticamente en las pruebas automatizadas del crate `dominio`, utilizando aritmética decimal exacta con `rust_decimal` (INV-03).

### Decisiones Arquitectónicas (ADRs)

Las decisiones técnicas de la Entrega 3 están documentadas en:

- **ADR-010 (Inmutabilidad de Movimientos):** Formaliza el patrón de Event Sourcing para movimientos, garantizando INV-04 e INV-13.
- **ADR-011 (Cálculo de Saldo con Aritmética Decimal Exacta):** Define que el saldo siempre se deriva de la suma de eventos con `rust_decimal`, cumpliendo INV-05 y RF-LIB-09.

### Ejecución de Pruebas Automatizadas (Entrega 3)

El proyecto incluye pruebas unitarias para los invariantes INV-04 e INV-05 y para cada validación de los casos de uso `RegistrarMovimiento` y `CorregirMovimiento`, ejecutadas con el repositorio en memoria (sin base de datos ni red), cumpliendo con los criterios de aceptación de la Entrega 3.

Para ejecutar las pruebas del núcleo Rust:

```bash
cargo test -p contrato -p aplicacion -p dominio
```

---

## Contribuciones

1. Crear una rama a partir de `main` para cada cambio.
2. Mantener el alcance del cambio claro y actualizar la documentación cuando corresponda.
3. Ejecutar las verificaciones relacionadas con los archivos modificados.
4. Abrir un pull request hacia `main`, describiendo el cambio y las comprobaciones realizadas.
5. No incluir archivos `.env`, credenciales ni artefactos de compilación en los commits.

---

## Licencia

Este proyecto se distribuye bajo la licencia Apache 2.0. Consulta el archivo `LICENSE` para conocer sus términos.
