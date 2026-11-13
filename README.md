# Cuentas Claras

Aplicación de gestión de finanzas personales con una interfaz web y una base multiplataforma construida con Tauri. Entrega del Grupo 2 — Sección 1, desarrollado para el curso CI3715, período septiembre–diciembre de 2026.

## Descripción

Cuentas Claras busca facilitar el registro de cuentas y movimientos financieros, la conversión de monedas y la importación y exportación del libro personal entre dispositivos. Su diseño contempla una aplicación de escritorio y Android, junto con un servidor para coordinar información compartida entre usuarios.

### Principios de manejo de datos

- **Aplicación local:** almacenar los datos financieros y personales en el dispositivo del usuario. Su exposición requiere autorización de la persona.
- **Servidor:** gestionar la información necesaria para las funciones compartidas, como amistades, solicitudes y grupos.

Estos principios describen el diseño previsto del producto; las funcionalidades se incorporan de forma incremental.

## Estado actual

| Área | Implementación disponible |
| --- | --- |
| Interfaz | Pantalla de estado de conexión, listado y creación de cuentas, detalle de cuenta con saldo, formulario para registrar/corregir movimientos, gestión de amistades, grupos y gastos compartidos. |
| Frontend | Next.js, React, TypeScript; exportación estática para Tauri. |
| Aplicación nativa | Tauri administra persistencia financiera local y comandos HTTP de autenticación; el token de sesión se conserva en el almacén seguro del sistema (`keyring`). |
| Backend | FastAPI con PostgreSQL. Endpoints de autenticación (`/usuarios`, `/sesiones`), amistades, grupos y gastos compartidos. |
| Gestión financiera | **Entregas 3, 4, 5 y 6 completadas:** Registro de movimientos con ajustes inmutables, manejo de tasas de cambio y patrimonio consolidado, autenticación segura con JWT, y división exacta de gastos en grupos con pruebas basadas en propiedades. |

El indicador «Conectado» de la interfaz no verifica todavía la disponibilidad real del servidor en todas las rutas, pero el núcleo de la aplicación funciona de forma autónoma.

---

## Entrega 6: Grupos y División de Gastos

### Casos de Uso Implementados
- **Amistades:** Búsqueda exacta de usuarios, envío y respuesta a invitaciones, y eliminación de amistad.
- **Grupos:** Creación de grupos, invitación de amigos y gestión de miembros (administrador/miembro).
- **Gastos Compartidos:** Registro de gastos con tres reglas de división: partes iguales, porcentajes y montos fijos.

### Invariantes Validados
- **INV-07 (Suma Exacta):** La suma de las partes de un gasto es exactamente el monto total. Ningún céntimo se pierde ni se inventa.
- **INV-08 (Balance Cero):** La suma de los balances de todos los miembros del grupo es exactamente cero en todo momento.

### Ejemplos de la Especificación (Secciones 2.4 y 2.5)

**1. Residuo del Redondeo (Sección 2.4)**  
Gasto de 100,00 en partes iguales entre `ana`, `beto` y `carla`. El residuo de 0,01 se reparte en orden alfabético:

| Participante | Parte Truncada | Residuo | Parte Final |
| --- | --- | --- | --- |
| ana | 33,33 | + 0,01 | **33,34** (incluye redondeo) |
| beto | 33,33 | 0,00 | **33,33** |
| carla | 33,33 | 0,00 | **33,33** |
| **Suma** | 99,99 | 0,01 | **100,00** |

**2. Balance del Grupo (Sección 2.5)**  
`ana` paga 100,00 (partes iguales) y `beto` paga 60,00 (ana 50%, beto 25%, carla 25%):

| Miembro | Pagó | Le corresponde | Balance |
| --- | --- | --- | --- |
| ana | 100,00 | 33,34 + 30,00 = 63,34 | **+36,66** |
| beto | 60,00 | 33,33 + 15,00 = 48,33 | **+11,67** |
| carla | 0,00 | 33,33 + 15,00 = 48,33 | **−48,33** |
| **Suma** | **160,00** | **160,00** | **0,00** |

### Decisiones Arquitectónicas (ADRs)
- **ADR-015:** Base de Datos del Servidor (PostgreSQL).
- **ADR-016:** Separación entre Datos Locales y Compartidos (INV-16, RES-02).
- **ADR-017:** Lógica de División de Gastos con Crate Independiente y pruebas basadas en propiedades (`proptest`).

### Ejecución de Pruebas Automatizadas (Entrega 6)
El crate `division` contiene la lógica pura de división y 18 pruebas automatizadas (unitarias y basadas en propiedades) que verifican INV-07 e INV-08 para cualquier combinación de datos.

```bash
cd app
cargo test -p division
```

---

## Entrega 5: Registro e Inicio de Sesión

La autenticación es opcional: solo identifica la cuenta ante el servidor. Las cuentas financieras, movimientos, saldos y tasas siguen en el SQLite local y no se envían con las peticiones de autenticación. La API guarda contraseñas con Argon2id y emite tokens JWT con 60 minutos de vigencia.

Para desarrollo, copia `.env.example` como `.env` en la raíz y configura `JWT_SECRET` con un valor generado localmente. No subas `.env` ni compartas ese secreto. La nota sobre almacenamiento de sesión en Android está en [`docs/deuda-tecnica.md`](docs/deuda-tecnica.md).

Para probar la API desde `servidor`:

```bash
cd servidor
python -m pip install -e ".[qa-auth]"
python -m uvicorn cuentas_claras.app.main:app --reload --app-dir src
```

La aplicación de escritorio lee `SERVER_HOST_IP` durante la compilación de Rust; esa variable debe estar en el entorno de `pnpm run tauri dev`. En Android Emulator, el host suele ser `[http://10.0.2.2:8000](http://10.0.2.2:8000)`; en escritorio local, `http://localhost:8000`.

---

## Entrega 4: Monedas, Tasas y Patrimonio

La Entrega 4 integra registro, historial y eliminación lógica de tasas; selección de moneda base; consulta histórica de patrimonio; y creación de monedas propias. Los movimientos se conservan en su moneda y no se convierten al cambiar la base.

### Ejemplo de la Especificación (Sección 2.3)
El sistema reproduce exactamente el flujo de consolidación de patrimonio:

| Cuenta | Saldo | Tasa usada | En VES |
| --- | --- | --- | --- |
| Efectivo bolívares | Bs. 88.700,00 | — | 88.700,00 |
| Banco en dólares | $ 200,00 | 1 USD = 887,00 VES (20/10, BCV) | 177.400,00 |
| Billetera USDT | USDT 50,00 | 1 USDT = 1.000,00 VES (21/10, casa de cambio) | 50.000,00 |
| **Total con moneda base VES** | | | **316.100,00** |

Si el usuario cambia la moneda base a USD: el efectivo se convierte con la inversa (88.700 ÷ 887 = 100,00 USD), pero como no existe tasa entre USDT y USD, el sistema **oculta el total consolidado** y muestra el aviso: *"Falta una tasa de USDT a USD con fecha igual o anterior al 23/10/2026"*. Al registrar la tasa faltante (1 USDT = 1,00 USD), el total pasa a ser **350,00 USD**.

### Ejecución de Pruebas Automatizadas (Entrega 4)
```bash
cd app
# Pruebas de dominio y casos de uso (incluyen tasas, patrimonio y monedas propias)
cargo test -p contrato -p dominio -p aplicacion
# Adaptadores SQLite, historial, eliminación y moneda propia
cargo test -p infraestructura --test repositorio_finanzas_sqlite --test repositorio_preferencias_sqlite
```

---

## Tecnologías y arquitectura

| Componente | Tecnologías | Responsabilidad |
| --- | --- | --- |
| Interfaz | Next.js · React · TypeScript | Presentación e interacción con el usuario. |
| Contenedor nativo | Tauri · Rust | Ejecución de la interfaz como aplicación nativa. |
| Servidor | FastAPI · Python · Uvicorn · PostgreSQL | API y coordinación de información compartida. |

El backend se organiza siguiendo una arquitectura hexagonal: dominio, aplicación, infraestructura, contratos y arranque de la API. El proyecto incluye configuración de Import Linter para revisar las dependencias entre capas y reglas de ESLint para organizar las dependencias del frontend.

```text
Grupo2_LimitTokenReached/
├── app/
│   ├── src/app/                 # Páginas, estilos y layout del frontend
│   ├── src-tauri/               # Proyecto nativo y configuración de Tauri
│   ├── crates/                  # Crates de Rust (dominio, aplicacion, infraestructura, contrato, division)
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
- Para el servidor: PostgreSQL local ejecutándose.

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

La configuración se lee desde `.env` en la raíz del repositorio. Los comandos se ejecutan desde `servidor`:

#### Windows — PowerShell

```powershell
cd servidor
python -m venv .venv
.\.venv\Scripts\python.exe -m pip install -e ".[qa-auth]"
.\.venv\Scripts\python.exe -m uvicorn cuentas_claras.app.main:app --reload --app-dir src
```

#### Linux / macOS

```bash
cd servidor
python3 -m venv .venv
.venv/bin/python -m pip install -e ".[qa-auth]"
.venv/bin/python -m uvicorn cuentas_claras.app.main:app --reload --app-dir src
```

Con el servidor en ejecución:

| Recurso | Dirección |
| --- | --- |
| Documentación interactiva | `[http://127.0.0.1:8000/docs](http://127.0.0.1:8000/docs)` |
| Esquema OpenAPI | `[http://127.0.0.1:8000/openapi.json](http://127.0.0.1:8000/openapi.json)` |
| Conexión | `GET /ping` |
| Registro | `POST /usuarios` |
| Inicio de sesión | `POST /sesiones` |
| Perfil autenticado | `GET /usuarios/yo` |

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

> **Pendientes en el código actual:** conectar la interfaz, que todavía simula la verificación, con `invoke('ping_servidor')`; ajustar el texto del botón al solicitado; cambiar el mensaje del backend de "Servidor de Cuentas Claras activo" a "Servidor Cuentas Claras activo"; y sustituir la llamada directa a `datetime.now()` por el puerto de reloj y su adaptador.

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

El servidor incluye herramientas de pruebas y revisión: Pytest, Ruff, Black, Mypy e Import Linter. Su configuración se encuentra en `servidor/pyproject.toml`.

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

### Núcleo Rust (Entregas 3, 4 y 6)

El proyecto incluye pruebas unitarias y basadas en propiedades para los invariantes y validaciones de dominio. Desde `app/`:

```bash
# Ejecutar todas las pruebas del núcleo Rust (contratos, aplicación, dominio y división)
cargo test -p contrato -p aplicacion -p dominio -p division
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
- **`RegistrarMovimiento`:** Permite anotar un ingreso o gasto en una cuenta, validando monto, categoría, fecha y moneda.
- **`CorregirMovimiento`:** Permite corregir un movimiento existente registrando un ajuste inmutable que referencia al original (INV-04).
- **`CrearCategoria`:** Permite al usuario definir nuevas categorías de gasto o ingreso.

### Invariantes Validados
- **INV-04 (Inmutabilidad de Movimientos):** Un movimiento registrado nunca se modifica ni se elimina. Las correcciones se registran como nuevos eventos de ajuste. La tabla de eventos solo admite `INSERT`, nunca `UPDATE` ni `DELETE`.
- **INV-05 (Cálculo de Saldo):** El saldo mostrado siempre coincide con `saldo inicial + ingresos − gastos ± ajustes`. Cualquier discrepancia es un defecto de severidad máxima (RF-LIB-09).

### Ejemplo de Corrección de Movimiento (Sección 2.2 de la Especificación)
El siguiente ejemplo reproduce el flujo de corrección exigido por la rúbrica. Cuenta "Billetera dólares" en USD:

| Paso | Movimiento | Efecto | Saldo |
| --- | --- | --- | --- |
| 1 | Saldo inicial | — | 1.000,00 |
| 2 | Ingreso · Salario | + 800,00 | 1.800,00 |
| 3 | Gasto · Alimentación (era 50, se anotó 500) | − 500,00 | 1.300,00 |
| 4 | Ajuste del paso 3 (monto correcto: 50,00) | + 450,00 | 1.750,00 |

El paso 3 sigue en el historial marcado como corregido, y el saldo final es el correcto: `1.000 + 800 − 50 = 1.750`.

Este ejemplo se reproduce matemáticamente en las pruebas automatizadas del crate `dominio`, utilizando aritmética decimal exacta con `rust_decimal` (INV-03).

### Decisiones Arquitectónicas (ADRs)
- **ADR-010 (Inmutabilidad de Movimientos):** Formaliza el patrón de Event Sourcing para movimientos, garantizando INV-04 e INV-13.
- **ADR-011 (Cálculo de Saldo con Aritmética Decimal Exacta):** Define que el saldo siempre se deriva de la suma de eventos con `rust_decimal`, cumpliendo INV-05 y RF-LIB-09.

---

## Contribuciones

Los PR del mismo repositorio hacia `main` pueden integrarse automáticamente cuando los jobs del CI pasan. La [guía de auto-merge](docs/ci-auto-merge.md) explica la configuración requerida en GitHub y cómo comprobar el flujo.

1. Crear una rama a partir de `main` para cada cambio.
2. Mantener el alcance del cambio claro y actualizar la documentación cuando corresponda.
3. Ejecutar las verificaciones relacionadas con los archivos modificados.
4. Abrir un pull request hacia `main`, describiendo el cambio y las comprobaciones realizadas.
5. No incluir archivos `.env`, credenciales ni artefactos de compilación en los commits.

---

## Licencia

Este proyecto se distribuye bajo la licencia Apache 2.0. Consulta el archivo `LICENSE` para conocer sus términos.
