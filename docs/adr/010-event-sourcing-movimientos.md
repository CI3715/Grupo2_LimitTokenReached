# ADR-010: Event Sourcing para Movimientos y Categorías

**Fecha:** 2026-10-20

**Estado:** Aceptado

**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached

**Etiquetas:** arquitectura, local-first, event-sourcing, inmutabilidad, INV-04, INV-13

## Contexto
La Entrega 3 requiere que los movimientos financieros (ingresos, gastos, ajustes) sean "imborrables" (INV-04). Un movimiento registrado nunca se modifica ni se elimina. Si el usuario se equivoca, el error se corrige con un nuevo movimiento de ajuste que referencia al original, igual que en un libro contable real. Además, el sistema debe ser 100% Local-First (sin internet).

## Decisión
Se extiende el patrón de Event Sourcing (definido en ADR-005) para incluir tres nuevos tipos de eventos inmutables en la tabla de eventos de SQLite:

1. `CategoriaCreada`: cuando el usuario define una nueva categoría.
2. `MovimientoRegistrado`: cuando se anota un ingreso o gasto.
3. `MovimientoCorregido`: cuando se aplica un ajuste a un movimiento existente.

La tabla de eventos en SQLite **solo admitirá operaciones `INSERT`**. El código del dominio e infraestructura tiene prohibido ejecutar `UPDATE` o `DELETE` sobre esta tabla.

Cada evento tiene un identificador único (UUID). Aplicar dos veces el mismo evento produce el mismo estado final (INV-13, idempotencia).

## Consecuencias
- **Positivas:** historial contable 100% auditable y a prueba de manipulaciones (INV-04). Idempotencia garantizada. Trazabilidad completa: cada corrección referencia al movimiento original.
- **Negativas:** requiere que el cálculo del saldo se haga proyectando todos los eventos, lo que podría ser lento si hay miles de movimientos (mitigado en ADR-011).

## Verificación
- No existe código en el crate `infraestructura` que ejecute `UPDATE` o `DELETE` en la tabla de eventos.
- Las pruebas de integración verifican que un intento de "borrar" un movimiento resulta en un nuevo evento `MovimientoCorregido` con monto compensatorio.
