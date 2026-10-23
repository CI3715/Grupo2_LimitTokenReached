# ADR-005: Estrategia Local-First con Event Sourcing y SQLite

**Fecha:** 2026-10-14  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** arquitectura, local-first, event-sourcing, SQLite, persistencia, privacidad

## Contexto
Se exige que las cuentas del usuario (efectivo, banco, billetera digital) se guarden **localmente en el dispositivo**, funcionando sin conexión a internet y con el servidor apagado (RNF-03, RNF-17). 

Además, también es indispensable cumplir lo siguiente: 
1. Los datos persistan al cerrar y reabrir la aplicación.
2. Se utilice SQLite embebido vía `rusqlite` dentro del `app_data_dir` de Tauri.
3. Crear una cuenta registre un **evento inmutable `CuentaCreada`** en un libro de eventos.
4. La lista de cuentas se obtenga a partir de esos eventos.
5. Ninguna operación sobre cuentas genere tráfico HTTP (INV-16, RES-02).

## Decisión
Se implementará un sistema **Local-First con Event Sourcing** utilizando:
- **Persistencia:** SQLite embebido mediante la crate `rusqlite` (síncrona), almacenando el archivo `.sqlite` en el `app_data_dir` nativo de Tauri.
- **Modelo de datos:** una tabla de eventos inmutables donde cada fila representa un evento `CuentaCreada` (con ID único, tipo, timestamp, dispositivo y payload).
- **Proyección:** el estado actual (lista de cuentas y saldos) se deriva de la lectura y proyección de estos eventos.
- **Aislamiento:** el servidor FastAPI de la Entrega 1 no participa en este flujo; solo responde a `/ping`.

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| Base de datos en el servidor (Postgres) | Viola el requisito de funcionamiento offline (RNF-03) y la privacidad de los datos (INV-16). |
| Archivos JSON o CSV planos | No garantiza integridad transaccional ni permite queries estructuradas de forma segura. |
| IndexedDB (lado web) | No es accesible de forma nativa y segura desde el núcleo Rust de Tauri. |
| Estado en memoria sin persistencia | Viola el requisito de que las cuentas sigan ahí al cerrar y reabrir la app. |

## Consecuencias

### Positivas
- **Autonomía total:** la aplicación funciona 100% offline (Prueba de Autonomía).
- **Privacidad por diseño:** los saldos y monedas nunca salen del dispositivo (INV-16).
- **Trazabilidad e Idempotencia:** al ser eventos inmutables, aplicar el mismo evento dos veces deja el mismo estado (INV-13).
- **Sin dependencias externas:** no requiere que el usuario instale motores de BD adicionales (RNF-12).

### Negativas / Restricciones
- No hay sincronización entre dispositivos en esta entrega (fuera de alcance).
- Si el usuario desinstala la app o pierde el dispositivo, los datos se pierden.

## Verificación (QA / CI)
- [x] La aplicación abre, crea cuentas y lista con el WiFi desactivado y el servidor Python apagado.
- [x] Tras cerrar y reabrir la aplicación, las cuentas persisten.
- [x] El archivo SQLite existe físicamente en el `app_data_dir` de Tauri.
- [x] La tabla de eventos contiene registros inmutables de tipo `CuentaCreada`.
- [x] Ninguna operación de crear/listar cuentas genera tráfico de red (verificado con herramientas de monitoreo).

## Referencias
- Especificación Técnica Entrega 2: Sección 2.2 (Almacenamiento Local) y 2.3 (Flujo Completo).
- Tauri Documentation: `app_data_dir` API.
- `rusqlite` crate documentation.
