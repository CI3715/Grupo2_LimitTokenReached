# ADR-006: Manejo de Dinero con rust_decimal (Prohibición de f32/f64)

**Fecha:** 2026-10-13  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** dominio, dinero, precisión, rust_decimal, INV-03, INV-01

## Contexto
El invariante **INV-03** prohíbe explícitamente usar `f32` o `f64` para representar dinero, debido a la pérdida de precisión inherente a la coma flotante binaria (ej. `0.1 + 0.2 = 0.30000000000000004`).

Además, el **INV-01** establece que todo monto es un par indivisible (cantidad + moneda), y cada moneda del catálogo base (VES, USD, EUR, USDT) tiene un número específico de decimales permitidos (2 decimales). El sistema debe rechazar saldos con más decimales que los permitidos por su moneda.

## Decisión
Se utilizará la crate **`rust_decimal`** para todos los cálculos y representaciones monetarias en el núcleo de Rust.
- **Internamente:** el saldo se maneja como `rust_decimal::Decimal`.
- **En los DTOs (Frontera con Next.js):** el saldo se serializa y deserializa como `String` para preservar la precisión exacta al viajar por JSON, evitando cualquier conversión implícita a `f64`.

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| `f64` (coma flotante estándar) | Pérdida de precisión en cálculos financieros. Viola explícitamente el INV-03. |
| `i64` (enteros, almacenando centavos) | Funciona, pero complica el manejo dinámico de monedas con diferentes cantidades de decimales. |
| `bigdecimal` (otra crate de Rust) | Más pesada y con mayor overhead de compilación que `rust_decimal` para este caso de uso. |

## Consecuencias

### Positivas
- **Precisión absoluta:** garantiza que `10.50 USD` sea exactamente eso, sin errores de redondeo.
- **Validación estricta:** permite rechazar fácilmente entradas como `10.555` para USD (que solo permite 2 decimales).
- **Seguridad en la frontera:** al usar `String` en los DTOs, Next.js recibe el valor exacto para formatearlo con `Intl.NumberFormat`.

### Negativas / Restricciones
- Requiere convertir explícitamente entre `String` y `Decimal` en los adaptadores de entrada/salida.
- `rust_decimal` no soporta operaciones trigonométricas (no necesarias para este dominio).

## Verificación (QA / CI)
- [ ] No existe ningún uso de `f32` o `f64` en los módulos de dominio relacionados con saldos o montos.
- [ ] Las pruebas unitarias validan que `10.555` es rechazado para USD.
- [ ] Los DTOs (`CuentaDTO`, `CrearCuentaRequest`) definen el campo `saldo` como `String`.
- [ ] El CI de Rust compila sin errores de tipos en el dominio.

## Referencias
- Especificación Técnica Entrega 2: Sección 2.1 (Reglas de validación).
- Invariante INV-03 (Prohibición de coma flotante para dinero).
- `rust_decimal` crate documentation.
