# ADR-013: Patrimonio por fecha y conversión decimal

**Fecha:** 2026-10-27
**Estado:** Propuesto; implementación y revisión del equipo pendientes
**Alcance:** Entrega 4, sección 2.3; INV-01/02/03, RF-CAM-04 y RF-INF-01

## Contexto

El patrimonio necesita combinar saldos de distintas monedas sin inventar equivalencias. La consulta debe explicar las tasas usadas y conservar los subtotales si no puede consolidar. Complementa [ADR-006](006-rust-decimal-dinero.md) y [ADR-011](011-calculo-saldo.md).

## Decisión propuesta

1. Recibir una fecha de consulta (hoy por defecto en la UI) y reconstruir saldos a esa fecha, incluyendo solo las cuentas y movimientos que correspondan al corte. Agrupar por moneda sin sumar monedas distintas.
2. Para cada moneda distinta de la base, buscar tasas activas directas hacia la base con fecha menor o igual al corte. Elegir la de fecha más reciente; ante empate, la última registrada según un orden estable del libro.
3. Si no hay directa aplicable, buscar una tasa aplicable en sentido contrario con el mismo criterio y convertir mediante su inversa. No construir rutas mediante una tercera moneda.
4. Calcular con `rust_decimal::Decimal`, preservando la precisión disponible durante conversiones y suma. En sentido inverso se puede dividir el saldo por el valor original para evitar truncar primero un recíproco. Las divisiones periódicas están limitadas por la precisión de `Decimal`; no se promete precisión infinita.
5. Redondear el total una sola vez, a los decimales de la base. La especificación no define cómo resolver empates de redondeo: el equipo debe acordar y probar esa política antes de cerrar la implementación.
6. Si falta una tasa, devolver `total_consolidado: None`, conservar saldos/subtotales y describir cada par faltante y fecha límite. La UI oculta el total y cualquier valor consolidado anterior.
7. Si la consolidación es posible, devolver el total como texto y las tasas usadas con valor, fecha, fuente e indicación de inversa. Mantener trazable el valor original cuando se aplique la inversa; acordar esa representación entre caso de uso y UI.

## Alternativas

- Considerar USDT equivalente a USD automáticamente: incumple INV-02.
- Convertir USDT→VES→USD: las tasas cruzadas están fuera del alcance.
- Redondear cada cuenta antes de sumar: puede producir un total distinto del redondeo final exigido.
- Calcular con `f32`/`f64`: contradice INV-03.
- Mostrar un total parcial cuando falta una tasa: presenta como patrimonio consolidado un valor incompleto.

## Consecuencias

El resultado es explicable y permite consultas pasadas. Se necesitan pruebas de dominio/casos de uso con repositorio fake y reloj controlado, además de las pruebas de DTOs. Las consultas deben responder a cambios de base o eliminación de tasas sin modificar el libro de movimientos.

## Verificación pendiente

- [ ] Ejemplo 2.3: 316.100,00 VES; sin total en USD hasta registrar USDT→USD; después 350,00 USD.
- [ ] Selección por corte, empate por orden de registro, exclusión de tasas eliminadas y ausencia de tasas cruzadas.
- [ ] Inversa y suma decimal con un único redondeo final; política de empate acordada.
- [ ] Saldos históricos excluyen movimientos y cuentas posteriores al corte.
- [ ] Falta de tasa impide consolidar en el caso de uso y en la UI.

## Estado observado

`ConsultarPatrimonioResponse` ya expresa el total opcional, tasas usadas y faltantes. Las pruebas de `patrimonio_tests.rs` construyen esas respuestas manualmente; no prueban todavía un cálculo ni un rechazo del dominio. La [guía de QA](../qa-entrega-4.md) detalla los escenarios necesarios.
