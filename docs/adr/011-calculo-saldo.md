# ADR-011: Cálculo de Saldo a partir de Movimientos (INV-05)

**Fecha:** 2026-10-20

**Estado:** Aceptado

**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached

**Etiquetas:** dominio, calculo, INV-05, rust_decimal

## Contexto
El invariante INV-05 y el requerimiento RF-LIB-09 exigen que el saldo de una cuenta sea siempre:

Saldo = Saldo Inicial + Ingresos − Gastos ± Ajustes

Cualquier discrepancia entre un saldo "precalculado" y esta suma es un defecto de severidad máxima.

## Decisión
El saldo mostrado en la UI **siempre** se derivará de la proyección de los eventos registrados.

- Se utilizará la crate `rust_decimal` para todas las operaciones de suma y resta, garantizando precisión exacta (INV-03).
- Por motivos de rendimiento en la UI, se permite mantener un campo `saldo_actual` precalculado en la entidad `Cuenta`, pero este campo **debe ser recalcado y validado** contra la suma de los movimientos cada vez que se registra o corrige un evento.
- Los ajustes (correcciones) se proyectan como eventos independientes que referencian al movimiento original, garantizando que el cálculo final coincida con el ejemplo de la sección 2.2 de la especificación.

## Consecuencias
- **Positivas:** imposibilidad de que el saldo se "desincronice" del historial real. Cumplimiento estricto de INV-05. El ejemplo de la sección 2.2 (saldo final 1.750,00 USD) se reproduce matemáticamente.
- **Negativas:** el cálculo requiere iterar sobre los movimientos de la cuenta. (Mitigación: Para el alcance de la Entrega 3, la suma en memoria con `rust_decimal` es suficiente. En futuras entregas se podría usar una tabla de proyección materializada).

## Verificación
- Pruebas unitarias que simulan el flujo del ejemplo de la sección 2.2 de la especificación, verificando que el saldo final sea matemáticamente exacto.
- Pruebas que inyectan un saldo precalculado incorrecto y verifican que el sistema lo detecta como defecto.
