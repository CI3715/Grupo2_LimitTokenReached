# ADR-010: Inmutabilidad de Movimientos (Event Sourcing)

**Fecha:** 2026-10-20 
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** arquitectura, local-first, event-sourcing, inmutabilidad, INV-04, INV-13

## Contexto
La especificación técnica de la Entrega 3 establece explícitamente:
> *"Un movimiento registrado nunca se modifica ni se elimina (INV-04). Si el usuario se equivocó, el error se corrige con un movimiento nuevo de ajuste que referencia al original, igual que en un libro contable, donde no se borra con corrector sino que se asienta la corrección."*

Además, la sección 2.4 exige:
> *"La tabla de eventos solo admite inserciones: el código no ejecuta UPDATE ni DELETE sobre ella (INV-04). Cada evento tiene un identificador único; aplicar dos veces el mismo evento deja el mismo estado (INV-13)."*

Esto busca garantizar que el historial contable sea 100% auditable, a prueba de manipulaciones y con trazabilidad completa de correcciones.

## Decisión
Se extiende el patrón de Event Sourcing (definido en ADR-005) para incluir tres nuevos tipos de eventos inmutables en la tabla de eventos de SQLite embebido:

1. **`CategoriaCreada`**: cuando el usuario define una nueva categoría.
2. **`MovimientoRegistrado`**: cuando se anota un ingreso o gasto.
3. **`MovimientoCorregido`**: cuando se aplica un ajuste a un movimiento existente, referenciando al original.

La capa de infraestructura (`infraestructura`) implementará **únicamente operaciones `INSERT`** sobre la tabla de eventos. Cualquier intento de ejecutar `UPDATE` o `DELETE` se considera una violación arquitectónica y será rechazado en revisión de código.

Cada evento tendrá un identificador único (UUID). El DTO `MovimientoDTO` incluirá los campos `es_correccion: bool` y `movimiento_original_id: Option<String>` para materializar la trazabilidad en la frontera del sistema.

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| Permitir `UPDATE` sobre movimientos | Viola INV-04. Rompe la auditoría contable y el principio de "libro contable imborrable". |
| Borrar físicamente el movimiento y reemplazarlo | Viola INV-04 y RF-LIB-06. El usuario no podría ver el historial real de lo ocurrido. |
| Usar una tabla separada de "correcciones" | Duplica lógica y complica el cálculo del saldo. Un solo flujo de eventos es más simple y coherente. |
| Corregir directamente el movimiento original | Viola la sección 2.2: *"Los ajustes no se corrigen directamente; siempre se corrige el movimiento original."* |

## Consecuencias

### Positivas
- **Historial 100% auditable:** cada corrección queda registrada como un nuevo evento con referencia al original.
- **Idempotencia garantizada (INV-13):** aplicar dos veces el mismo evento produce el mismo estado final.
- **Trazabilidad completa:** el campo `movimiento_original_id` permite reconstruir la cadena de correcciones.
- **Cumplimiento estricto de la rúbrica:** la sección 2.2 (ejemplo de la Billetera dólares) se reproduce matemáticamente.
- **Coherencia con ADR-005:** se reutiliza el patrón ya adoptado para `CuentaCreada`.

### Negativas / Restricciones
- El cálculo del saldo requiere proyectar todos los eventos (mitigado en ADR-011).
- Mayor volumen de datos en la tabla de eventos a largo plazo (no crítico para el alcance de la Entrega 3).
- La UI debe manejar la visualización de movimientos corregidos y sus ajustes.

## Verificación (QA / CI)
- [ ] No existe código en el crate `infraestructura` que ejecute `UPDATE` o `DELETE` sobre la tabla de eventos.
- [ ] Las pruebas automatizadas validan INV-04: un intento de "editar" un movimiento genera un nuevo evento `MovimientoCorregido`.
- [ ] Las pruebas automatizadas validan INV-13: aplicar dos veces el mismo evento deja el estado inalterado.
- [ ] El ejemplo de la sección 2.2 (saldo final 1.750,00 USD) se reproduce en las pruebas unitarias.
- [ ] El README explica cómo ejecutar estas pruebas con el repositorio en memoria.

## Referencias
- Especificación Técnica Entrega 3: Secciones 2.2 (Corregir un Movimiento) y 2.4 (Almacenamiento Local).
- Criterios de Aceptación: *"No hay ninguna forma de editar ni eliminar un movimiento."*
- ADR-005: Local-First y Event Sourcing.
- ADR-009: Pruebas de Dominio con Repositorio en Memoria.
