# Cuentas Claras

Aplicación de gestión de finanzas personales con una interfaz web y una base multiplataforma construida con Tauri. Proyecto del Grupo 2 — Sección 1, desarrollado para el curso CI3715, período septiembre–diciembre de 2026.

## Descripción

Cuentas Claras busca facilitar el registro de cuentas y movimientos financieros, la conversión de monedas y la importación y exportación del libro personal entre dispositivos. Su diseño contempla una aplicación de escritorio y Android, junto con un servidor para coordinar información compartida entre usuarios.

### Principios de manejo de datos

- **Aplicación local:** almacenar los datos financieros y personales en el dispositivo del usuario. Su exposición requiere autorización de la persona.
- **Servidor:** gestionar la información necesaria para las funciones compartidas, como amistades, solicitudes, grupos y gastos compartidos.

Estos principios describen el diseño previsto del producto; las funcionalidades se incorporan de forma incremental.

## Estado actual

| Área | Implementación disponible |
| --- | --- |
| Interfaz | Estado de conexión, listado y creación de cuentas, detalle con saldo y movimientos, registro y corrección de movimientos, tasas, moneda base, patrimonio, y pantallas de registro, inicio de sesión, amistades, grupos, gastos compartidos y sincronización. |
| Frontend | Next.js, React, TypeScript, `react-hook-form`, `zod` y `decimal.js`; exportación estática para Tauri. |
| Aplicación nativa | Tauri administra persistencia financiera local (SQLite) y comandos HTTP de autenticación y funciones sociales; el token de sesión se conserva en el almacén seguro del sistema (`keyring`) o en archivo privado en Android (ver deuda técnica). |
| Backend | FastAPI con SQLAlchemy y **SQLite**. Endpoints de autenticación, amistades, grupos, gastos compartidos y sincronización (`GET /cambios`). Contraseñas con Argon2 y tokens JWT. |
| Gestión financiera | Entregas 2 a 7 completadas: cuentas, movimientos inmutables, tasas, patrimonio, autenticación, división exacta de gastos, y sincronización idempotente sin duplicados. |

El indicador «Conectado» de la interfaz no verifica todavía la disponibilidad real del servidor en todas las rutas, pero el núcleo de la aplicación funciona de forma autónoma.

## Tecnologías y arquitectura

| Componente | Tecnologías | Responsabilidad |
| --- | --- | --- |
| Interfaz | Next.js · React · TypeScript · `react-hook-form` · `zod` · `decimal.js` | Presentación, validación de formularios y aritmética decimal en el cliente. |
| Contenedor nativo | Tauri · Rust | Ejecución como aplicación nativa, persistencia local y puente HTTP. |
| Servidor | FastAPI · Python · Uvicorn · SQLAlchemy · `argon2-cffi` · `PyJWT` | API, autenticación, registro de cambios y coordinación de información compartida. |

El backend en Rust se organiza siguiendo una arquitectura hexagonal con crates separados: `dominio`, `aplicacion`, `infraestructura`, `contrato` y `division`. El proyecto incluye configuración de Import Linter para revisar las dependencias entre capas del servidor y reglas de ESLint con `eslint-plugin-boundaries` para organizar las dependencias del frontend.

```text
Grupo2_LimitTokenReached/
├── app/
│   ├── src/app/                 # Páginas, estilos y layout del frontend
│   ├── src-tauri/               # Proyecto nativo y configuración de Tauri
│   ├── crates/
│   │   ├── contrato/            # DTOs y pruebas (cuenta, movimiento, tasa, patrimonio, auth, social, sincronizacion)
│   │   ├── dominio/             # Reglas de negocio en Rust
│   │   ├── aplicacion/          # Casos de uso y puertos
│   │   ├── infraestructura/     # Adaptadores SQLite y HTTP
│   │   └── division/            # Lógica pura de división de gastos (INV-07, INV-08)
│   ├── public/                  # Recursos estáticos
│   └── package.json             # Dependencias y scripts del frontend
├── servidor/
│   ├── src/cuentas_claras/
│   │   ├── dominio/             # Reglas del negocio (Python)
│   │   ├── aplicacion/          # Casos de uso y puertos
│   │   ├── infraestructura/     # Adaptadores (SQLAlchemy, seguridad)
│   │   ├── contrato/            # Contratos de la API (incluye sincronizacion.py)
│   │   └── app/                 # Configuración y arranque de FastAPI
│   ├── tests/                   # Pruebas del servidor
│   ├── pyproject.toml
│   └── requirements.txt
├── docs/
│   ├── adr/                     # Decisiones de arquitectura
│   └── deuda-tecnica.md         # Partidas técnicas registradas (ej. token en Android)
└── LICENSE
```

## Desarrollo local

### Requisitos

- Git.
- Node.js y pnpm. El frontend se ha compilado localmente con Node.js 22.
- Python 3.11 o superior.
- Para ejecutar Tauri: Rust, Cargo y las dependencias nativas correspondientes al sistema operativo.

### Obtener el proyecto

```bash
git clone [https://github.com/CI3715/Grupo2_LimitTokenReached.git](https://github.com/CI3715/Grupo2_LimitTokenReached.git)
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

> **Nota:** en PowerShell, si la política de ejecución bloquea `pnpm.ps1`, utiliza `pnpm.cmd` en lugar de `pnpm`.

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
| Documentación interactiva | `http://127.0.0.1:8000/docs` |
| Esquema OpenAPI | `http://127.0.0.1:8000/openapi.json` |
| Conexión | `GET /ping` |
| Registro | `POST /usuarios` |
| Inicio de sesión | `POST /sesiones` |
| Perfil autenticado | `GET /usuarios/yo` |
| Obtener cambios | `GET /cambios?desde=<cursor>` |

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

### Ejecutar la aplicación de escritorio

Desde `app/`, con las dependencias nativas de Tauri instaladas:

```bash
pnpm run tauri dev
```

Para generar el paquete nativo:

```bash
pnpm run tauri build
```

El comando Rust utiliza por defecto `http://localhost:8000` como URL base y añade `/ping`. Si se necesita otra dirección, `SERVER_HOST_IP` debe contener la URL base completa, sin `/docs` ni `/ping`, y estar definida en el entorno al compilar o ejecutar `pnpm run tauri dev`: el código Rust la lee en compilación mediante `option_env!`. Un archivo `.env` del backend no configura automáticamente esta variable para Tauri. En Android Emulator, el host de la computadora suele ser `http://10.0.2.2:8000`.

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

Para comprobar las dependencias entre capas con Import Linter:

```bash
# Linux / macOS
PYTHONPATH=src lint-imports
```

```powershell
# Windows — PowerShell
$env:PYTHONPATH = "src"
lint-imports
```

## Núcleo Rust: contratos, dominio y división

El crate `contrato` agrupa los DTOs y sus pruebas por área: `crear_cuenta_tests`, `movimiento_tests`, `tasa_tests`, `patrimonio_tests`, `autenticacion_tests`, `social_tests` y `sincronizacion_tests`. El crate `division` contiene la lógica pura de división de gastos con pruebas unitarias y basadas en propiedades (`proptest`).

Desde `app/`:

```bash
# Contratos, dominio y casos de uso
cargo test -p contrato -p aplicacion -p dominio

# Lógica de división: INV-07, INV-08 y regla de residuo (unitarias + property-based)
cargo test -p division

# Adaptadores SQLite locales (sin red ni servidor)
cargo test -p infraestructura --test repositorio_movimientos_sqlite --test repositorio_categorias_sqlite --test repositorio_monedas_sqlite

# Adaptador HTTP de autenticación
cargo test -p infraestructura --test autenticacion_http
```

---

## Creación de Cuentas y Almacenamiento Local (Entrega 2)

### Almacenamiento
Las cuentas se guardan localmente en el dispositivo usando SQLite embebido (`rusqlite`). El archivo de base de datos se almacena en el directorio de datos nativo de la aplicación (`app_data_dir` de Tauri), garantizando que los datos persistan al cerrar y reabrir la app, y que no salgan del dispositivo (Local-First).

### Pruebas
Pruebas unitarias para los invariantes de creación de cuentas (INV-01, INV-03, INV-18) y validaciones de `CrearCuenta`, ejecutadas con un repositorio en memoria (sin base de datos real ni red).

---

## Registrar Movimientos y Ver Saldos (Entrega 3)

### Casos de Uso Implementados
- **`RegistrarMovimiento`:** anota un ingreso o gasto en una cuenta, validando monto, categoría, fecha y moneda.
- **`CorregirMovimiento`:** corrige un movimiento existente registrando un ajuste inmutable que referencia al original (INV-04).
- **`CrearCategoria`:** define nuevas categorías de gasto o ingreso.

### Invariantes Validados
- **INV-04 (Inmutabilidad de Movimientos):** un movimiento registrado nunca se modifica ni se elimina; las correcciones son nuevos eventos de ajuste. La tabla de eventos solo admite `INSERT`.
- **INV-05 (Cálculo de Saldo):** el saldo coincide con `saldo inicial + ingresos − gastos ± ajustes`.

### Ejemplo de Corrección de Movimiento (Sección 2.2)
Cuenta "Billetera dólares" en USD:

| Paso | Movimiento | Efecto | Saldo |
| --- | --- | --- | --- |
| 1 | Saldo inicial | — | 1.000,00 |
| 2 | Ingreso · Salario | + 800,00 | 1.800,00 |
| 3 | Gasto · Alimentación (era 50, se anotó 500) | − 500,00 | 1.300,00 |
| 4 | Ajuste del paso 3 (monto correcto: 50,00) | + 450,00 | 1.750,00 |

El paso 3 queda en el historial marcado como corregido; el saldo final correcto es `1.000 + 800 − 50 = 1.750`. Se reproduce en las pruebas del crate `dominio` con `rust_decimal` (INV-03).

### Decisiones Arquitectónicas (ADRs)
- **ADR-010 (Inmutabilidad de Movimientos):** patrón de Event Sourcing para movimientos (INV-04, INV-13).
- **ADR-011 (Cálculo de Saldo con Aritmética Decimal Exacta):** el saldo se deriva de la suma de eventos con `rust_decimal` (INV-05, RF-LIB-09).

---

## Monedas, Tasas y Patrimonio (Entrega 4)

La Entrega 4 integra registro, historial y eliminación lógica de tasas; selección de moneda base; consulta histórica de patrimonio; y creación de monedas propias. Los movimientos se conservan en su moneda y no se convierten al cambiar la base.

### Ejemplo de la Especificación (Sección 2.3)

| Cuenta | Saldo | Tasa usada | En VES |
| --- | --- | --- | --- |
| Efectivo bolívares | Bs. 88.700,00 | — | 88.700,00 |
| Banco en dólares | $ 200,00 | 1 USD = 887,00 VES · 20/10 · BCV | 177.400,00 |
| Billetera USDT | USDT 50,00 | 1 USDT = 1.000,00 VES · 21/10 · casa de cambio | 50.000,00 |
| **Total con moneda base VES** | | | **316.100,00** |

Si el usuario cambia la base a USD: el efectivo se convierte con la inversa (88.700 ÷ 887 = 100,00 USD), pero como no existe tasa entre USDT y USD, el sistema **oculta el total consolidado** y avisa: *"Falta una tasa de USDT a USD con fecha igual o anterior al 23/10/2026"*. Al registrar 1 USDT = 1,00 USD, el total pasa a **350,00 USD**.

### Decisiones Arquitectónicas (ADRs)
- **ADR-012 (Eventos de tasas y monedas):** tasas y monedas como eventos inmutables (INV-04, INV-13).
- **ADR-013 (Patrimonio y precisión decimal):** selección de tasa vigente, inversa y redondeo final único (INV-02, INV-03).

---

## Registro e Inicio de Sesión (Entrega 5)

La autenticación es opcional: solo identifica la cuenta ante el servidor. Las cuentas financieras, movimientos, saldos y tasas siguen en el SQLite local y **no se envían** con las peticiones de autenticación (INV-16, RES-02).

- **Seguridad:** contraseñas con Argon2 (hash con sal única, nunca en texto plano, RNF-05) y tokens JWT firmados con vigencia de 60 minutos (RF-SOC-02, RES-07). El secreto se configura por variable de entorno.
- **Almacenamiento del token:** en el almacén de secretos del sistema (`keyring`); en Android se usa un archivo privado registrado como deuda técnica (ver `docs/deuda-tecnica.md`).
- **Autonomía:** sin conexión o con el servidor apagado, las funciones personales siguen funcionando; solo el registro y el inicio de sesión muestran que no hay conexión (RNF-17).

Para desarrollo, copia `.env.example` como `.env` en la raíz y configura `JWT_SECRET` y `DATABASE_URL` (ej. `sqlite:///./servidor.db`). Para el backend remoto, `DATABASE_URL` puede ser la URI PostgreSQL privada de Supabase y prevalece sobre `DATABASE_PATH`; no la incluyas en el frontend ni subas `.env` al repositorio.

### Decisiones Arquitectónicas (ADRs)
- **ADR-014 (Autenticación local-servidor):** JWT, Argon2 y almacenamiento seguro del token.
- **ADR-015 (Base de datos del servidor):** justificación del uso de SQLite en el backend para simplificar el despliegue y mantenimiento en el alcance académico.

---

## Grupos y División de Gastos (Entrega 6)

### Casos de Uso Implementados
- **Amistades:** búsqueda por nombre de usuario exacto (nunca devuelve el correo, RF-SOC-04), envío y respuesta a invitaciones, y eliminación de amistad.
- **Grupos:** creación de grupos, invitación de amigos y gestión de miembros (administrador/miembro).
- **Gastos Compartidos:** registro con tres reglas de división: partes iguales, porcentajes y montos fijos.

### Invariantes Validados
- **INV-07 (Suma Exacta):** la suma de las partes es exactamente el monto total; ningún céntimo se pierde ni se inventa.
- **INV-08 (Balance Cero):** la suma de los balances de todos los miembros es exactamente cero.

### Ejemplos de la Especificación (Secciones 2.4 y 2.5)

**1. Residuo del redondeo (Sección 2.4)** — Gasto de 100,00 entre `ana`, `beto` y `carla`; el residuo de 0,01 se asigna en orden alfabético:

| Participante | Parte truncada | Residuo | Parte final |
| --- | --- | --- | --- |
| ana | 33,33 | + 0,01 | **33,34** (incluye redondeo) |
| beto | 33,33 | 0,00 | **33,33** |
| carla | 33,33 | 0,00 | **33,33** |
| **Suma** | 99,99 | 0,01 | **100,00** |

**2. Balance del grupo (Sección 2.5)** — `ana` paga 100,00 (partes iguales) y `beto` paga 60,00 (ana 50%, beto 25%, carla 25%):

| Miembro | Pagó | Le corresponde | Balance |
| --- | --- | --- | --- |
| ana | 100,00 | 33,34 + 30,00 = 63,34 | **+36,66** |
| beto | 60,00 | 33,33 + 15,00 = 48,33 | **+11,67** |
| carla | 0,00 | 33,33 + 15,00 = 48,33 | **−48,33** |
| **Suma** | **160,00** | **160,00** | **0,00** |

### Decisiones Arquitectónicas (ADRs)
- **ADR-016 (Separación entre datos locales y compartidos):** lo personal no viaja al servidor (INV-16, RES-02).
- **ADR-017 (Lógica de división de gastos):** crate `division` con aritmética exacta y pruebas basadas en propiedades.

### Pruebas de división (Entrega 6)
El crate `division` contiene la lógica pura de división y pruebas que verifican INV-07 e INV-08 para cualquier combinación de datos, ejecutadas con repositorio en memoria (sin base de datos ni red):
```bash
cd app
cargo test -p division
```

---

## Sincronización y Gastos de Grupo (Entrega 7)

### Casos de Uso Implementados
- **Elección de cuenta:** el usuario selecciona una cuenta personal (en la misma moneda del grupo) donde se aplicarán sus partes.
- **Sincronización de bajada (Pull):** la app solicita eventos nuevos al servidor mediante `GET /cambios?desde=<cursor>`, recibiendo lotes de máximo 100 eventos.
- **Sincronización de subida (Push) con cola offline:** los gastos registrados sin conexión se guardan en una cola local y se envían al recuperar la red, utilizando un encabezado `Idempotency-Key` único para evitar duplicados por reintentos.

### Invariantes Validados
- **INV-13, INV-15 (Idempotencia):** aplicar el mismo evento o lote dos veces deja exactamente el mismo estado (cero duplicados).
- **INV-14 (Solo agrega):** la sincronización nunca borra ni modifica eventos locales.
- **INV-16 (Privacidad):** ninguna petición al servidor contiene cuentas, movimientos, saldos ni tasas del libro personal.

### Ejemplo de la Especificación (Sección 2.6)
Grupo “Viaje a Mérida” (USD). beto elige su cuenta **Efectivo USD**, con saldo inicial de 200,00:

| Momento | Lo que pasa | Saldo de Efectivo USD |
| --- | --- | --- |
| Inicio | beto elige la cuenta para el grupo. | 200,00 |
| Sincronización 1 | Baja su parte del hotel de 100,00 en partes iguales: gasto de 33,33. | 166,67 |
| Sincronizaciones 2 y 3 | No hay eventos nuevos. | 166,67 |
| Sin conexión | beto registra la cena de 60,00 (50% · 25% · 25%); queda pendiente de envío. | 166,67 |
| Vuelve la conexión | La cena se envía una vez y baja su parte: gasto de 15,00. | 151,67 |
| Cursor reiniciado a cero | Bajan de nuevo los dos eventos; ya estaban aplicados y se ignoran. | 151,67 |

### Decisiones Arquitectónicas (ADRs)
- **ADR-018 (Sincronización idempotente y basada en cursor):** documenta el mecanismo de `GET /cambios` con cursor para la bajada de eventos y el uso de `Idempotency-Key` en `POST /grupos/{id}/gastos` para la subida, garantizando cero duplicados y soporte offline robusto.

### Pruebas de Sincronización (Entrega 7)
El proyecto incluye pruebas automatizadas que validan:
- Idempotencia en cliente (aplicar el mismo lote dos veces o reiniciar cursor a cero sin duplicar).
- Interrupción a mitad de lote y recuperación sin duplicados.
- Cola offline con reintentos y rechazos.
- Idempotencia en servidor (verificar respuestas 200 vs 409 ante claves repetidas).

---

## Contribuciones

Los PR del mismo repositorio hacia `main` pueden integrarse automáticamente cuando los jobs del CI pasan. La [guía de auto-merge](docs/ci-auto-merge.md) explica la configuración requerida en GitHub y cómo comprobar el flujo.

1. Crear una rama a partir de `main` para cada cambio.
2. Mantener el alcance del cambio claro y actualizar la documentación cuando corresponda.
3. Ejecutar las verificaciones relacionadas con los archivos modificados.
4. Abrir un pull request hacia `main`, describiendo el cambio y las comprobaciones realizadas.
5. No incluir archivos `.env`, credenciales ni artefactos de compilación en los commits.

## Licencia

Este proyecto se distribuye bajo la licencia Apache 2.0. Consulta el archivo `LICENSE` para conocer sus términos.
