# ADR-011: Cálculo de Saldo con Aritmética Decimal Exacta

**Fecha:** 2026-10-20  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** dominio, calculo, INV-03, INV-05, rust_decimal, RF-LIB-09

## Contexto
La especificación técnica de la Entrega 3 establece explícitamente:
> *"Saldo actual = saldo inicial + ingresos − gastos ± ajustes. Se calcula siempre a partir de los movimientos (INV-05). Si se guarda un saldo precalculado para mostrarlo más rápido, debe coincidir siempre con esa suma; una diferencia es un defecto de severidad máxima (RF-LIB-09)."*

Además, la sección 2.1 reitera:
> *"Todo cálculo de dinero usa aritmética decimal exacta (rust_decimal); está prohibido f32 o f64 (INV-03)."*

Esto busca garantizar que el saldo mostrado al usuario sea matemáticamente exacto y coincida con la suma de los eventos registrados, sin errores de redondeo por coma flotante binaria.

## Decisión
El saldo mostrado en la UI **siempre** se derivará de la proyección de los eventos registrados, aplicando la siguiente fórmula:

Saldo = Saldo Inicial + Σ(Ingresos) − Σ(Gastos) ± Σ(Ajustes)

- Se utilizará la crate `rust_decimal` (ADR-006) para **todas** las operaciones de suma y resta, garantizando precisión exacta.
- Los DTOs (`CuentaDTO`, `MovimientoDTO`, `RegistrarMovimientoRequest`, `CorregirMovimientoRequest`) transportan los montos como `String` (INV-03), y el dominio los convierte a `Decimal` antes de cualquier cálculo.
- Por motivos de rendimiento en la UI, se permite mantener un campo `saldo_actual` precalculado en la entidad `Cuenta`, pero este campo **debe ser recalcado y validado** contra la suma de los movimientos cada vez que se registra o corrige un evento. Cualquier discrepancia se trata como un defecto de severidad máxima (RF-LIB-09).
- Los ajustes (correcciones) se proyectan como eventos independientes que referencian al movimiento original, garantizando que el cálculo final coincida con el ejemplo de la sección 2.2.

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| Usar `f64` para los cálculos | Viola INV-03. Introduce errores de redondeo binario (ej. `0.1 + 0.2 ≠ 0.3`). |
| Calcular el saldo solo en el Frontend | Rompe la arquitectura hexagonal. El dominio debe ser la fuente de verdad. |
| No validar el saldo precalculado contra la suma | Viola RF-LIB-09. Una discrepancia sería un defecto de severidad máxima no detectado. |
| Usar enteros (centavos) en lugar de `rust_decimal` | Funciona, pero `rust_decimal` es más expresivo y ya está adoptado en ADR-006. |

## Consecuencias

### Positivas
- **Precisión matemática garantizada:** imposibilidad de errores de redondeo binario (INV-03).
- **Cumplimiento estricto de INV-05 y RF-LIB-09:** el saldo siempre coincide con la suma de eventos.
- **El ejemplo de la sección 2.2 se reproduce tal cual:** saldo final de 1.750,00 USD tras la corrección.
- **Coherencia con ADR-006:** se reutiliza la crate `rust_decimal` ya adoptada en la Entrega 2.
- **Detección temprana de defectos:** si el saldo precalculado difiere de la suma, el sistema lo marca como defecto crítico.

### Negativas / Restricciones
- El cálculo requiere iterar sobre los movimientos de la cuenta (mitigación: para el alcance de la Entrega 3, la suma en memoria con `rust_decimal` es suficiente).
- Mayor complejidad en la conversión `String → Decimal` en el dominio (mitigado por las pruebas de INV-03).
- Requiere pruebas exhaustivas que reproduzcan el ejemplo de la sección 2.2.

## Verificación (QA / CI)
- [ ] Las pruebas automatizadas validan INV-05: el saldo calculado coincide con `inicial + ingresos − gastos ± ajustes`.
- [ ] Las pruebas automatizadas validan INV-03: ningún cálculo usa `f32` o `f64`.
- [ ] El ejemplo de la sección 2.2 (saldo final 1.750,00 USD) se reproduce exitosamente en las pruebas unitarias.
- [ ] Las pruebas detectan discrepancias entre el saldo precalculado y la suma de eventos (RF-LIB-09).
- [ ] El README explica cómo ejecutar estas pruebas con el repositorio en memoria.

## Referencias
- Especificación Técnica Entrega 3: Secciones 2.1 (Reglas de validación), 2.2 (Corregir un Movimiento) y 2.3 (Saldo y Detalle de la Cuenta).
- Criterios de Aceptación: *"El saldo mostrado coincide siempre con saldo inicial + ingresos − gastos ± ajustes."*
- ADR-006: Uso de `rust_decimal` para Manejo de Dinero.
- ADR-009: Pruebas de Dominio con Repositorio en Memoria.
- ADR-010: Inmutabilidad de Movimientos (Event Sourcing).
