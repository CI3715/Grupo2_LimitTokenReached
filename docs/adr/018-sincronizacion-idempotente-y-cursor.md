# ADR-018: Sincronización Idempotente y Basada en Cursor para Gastos de Grupo

**Fecha:** 2026-11-19  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** sincronizacion, idempotencia, offline, INV-13, INV-14, INV-15, INV-16, RNF-17, Entrega-7

## Contexto
La especificación de la Entrega 7 exige que cada gasto de grupo se refleje en el libro personal de cada participante como **exactamente un movimiento**, sin importar cuántas veces la aplicación se sincronice con el servidor (INV-13, INV-15). Además, los gastos registrados sin conexión deben enviarse de forma segura al recuperar la conectividad, evitando duplicados por reintentos de red (RNF-17, ESC-09). 

El servidor es la fuente de la verdad para los gastos de grupo, pero la aplicación decide localmente en qué cuenta personal aplicar la parte. El servidor nunca sabe qué cuentas tiene el usuario ni cuál es su saldo (INV-16, RES-02).

## Decisión
Se implementa un mecanismo de sincronización bidireccional basado en dos pilares:

### 1. Sincronización de Bajada (Pull) basada en Cursor
- El servidor mantiene un registro de cambios (log de eventos) inmutable y *append-only* por usuario.
- Cada evento tiene un identificador único (`id_evento`) y una posición creciente (`posicion_cursor`).
- El cliente solicita cambios mediante `GET /cambios?desde=<cursor>`.
- El servidor devuelve un máximo de 100 eventos por lote, junto con el `nuevo_cursor` y un flag `hay_mas`.
- El cliente aplica los eventos localmente (generando un evento `GastoDeGrupoAplicado`) y guarda el `nuevo_cursor` en la **misma transacción SQLite**. Si la app se cierra a mitad de camino, al volver pide desde el último cursor guardado y la comprobación por `id_evento` evita duplicados (INV-14).

### 2. Sincronización de Subida (Push) con Idempotencia
- El cliente genera una `Idempotency-Key` (UUID v4) única por cada gasto de grupo registrado en la cola sin conexión.
- Este UUID se envía en el encabezado HTTP `Idempotency-Key` en el `POST /grupos/{id}/gastos`.
- El servidor almacena temporalmente estas claves. Si recibe una clave ya procesada:
  - Si los datos coinciden: responde `200 OK` con el gasto existente (sin duplicar).
  - Si los datos difieren: responde `409 Conflict`.

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| Sincronización completa (descargar todo el historial) | Ineficiente y no escala. El enfoque por cursor es O(1) en memoria y solo transfiere lo nuevo. |
| El servidor empuja eventos (WebSockets/Push) | Requiere conexión constante, viola RNF-17 (la app debe funcionar offline y sincronizar al volver). |
| El cliente genera los IDs de los eventos | El servidor es la fuente de la verdad; debe generar el ID y el orden (cursor) para garantizar consistencia entre todos los miembros del grupo. |

## Consecuencias

### Positivas
- **Cero duplicados (INV-13, INV-15):** Garantizado tanto por el cursor en la bajada como por la `Idempotency-Key` en la subida.
- **Soporte Offline robusto (RNF-17):** La cola local con claves de idempotencia permite registrar gastos sin red y enviarlos después sin riesgo de duplicar.
- **Eficiencia de red:** Solo se transfieren lotes de 100 eventos nuevos, ideal para redes móviles inestables.
- **Privacidad (INV-16):** El servidor solo sabe qué gastos de grupo se registraron, nunca en qué cuenta personal se aplicaron ni cuál es el saldo del usuario.

### Negativas / Restricciones
- **Costo de almacenamiento en el servidor:** El servidor debe guardar las claves de idempotencia y el log de cambios. *Mitigación:* Las claves de idempotencia pueden tener un TTL (tiempo de vida) de 7 a 30 días, suficiente para cubrir reintentos de red.
- **Complejidad en el cliente:** La lógica de la cola offline y el manejo del cursor requiere una gestión cuidadosa de las transacciones SQLite para garantizar atomicidad.

## Verificación (QA / CI)
- [ ] Pruebas automatizadas de idempotencia en cliente (aplicar el mismo lote dos veces o reiniciar cursor a cero sin duplicar).
- [ ] Pruebas de interrupción a mitad de lote y recuperación sin duplicados.
- [ ] Pruebas de la cola offline con reintentos y rechazos.
- [ ] Pruebas de idempotencia en servidor (verificar respuestas 200 vs 409).
- [ ] Validación del flujo completo (The Sync Flow) verificando que ningún request envíe datos personales (cuentas, saldos, tasas).

## Referencias
- Especificación Técnica Entrega 7: Secciones 2.1, 2.2, 2.3, 2.4, 2.5 y 2.7.
- INV-13, INV-14, INV-15, INV-16, INV-17, INV-18.
- RNF-17 (Funcionamiento sin conexión), ESC-09.
- ADR-017: Lógica de División de Gastos con Crate Independiente.
