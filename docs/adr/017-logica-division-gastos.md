# ADR-017: Lógica de División de Gastos con Crate Independiente

**Fecha:** 2026-11-10  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** dominio, calculo, INV-07, INV-08, RNF-08, proptest, property-based-testing, Entrega-6

## Contexto
En la entrega 6 se indica que:
> *"Cuando la división no es exacta, ningún céntimo se pierde ni se inventa (INV-07, ESC-10). El procedimiento es determinista: con los mismos datos, siempre el mismo resultado."*

Además, la sección 2.5 exige:
> *"La suma de los balances de todos los miembros es exactamente cero en todo momento (INV-08)."*

Y los criterios de aceptación requieren:
> *"Hay pruebas automatizadas de la división (incluidos casos con residuo, un solo participante y montos grandes), pruebas basadas en propiedades que comprueban que las partes suman el total y los balances suman cero..."*

Esto busca garantizar que la aritmética de la división de gastos sea matemáticamente exacta, sin errores de redondeo, y que las propiedades matemáticas (INV-07, INV-08) se cumplan para **cualquier** combinación de datos, no solo para ejemplos concretos.

## Decisión
Se crea un **crate independiente en Rust llamado `division`** que contiene únicamente la lógica matemática pura de la división de gastos, sin dependencias de base de datos, red ni infraestructura.

**Características del crate:**
- Implementa las tres reglas de división: partes iguales, porcentajes y montos fijos.
- Usa `rust_decimal` en toda la aritmética (cumple RNF-08: prohibido coma flotante binaria).
- Implementa la regla determinista de residuo: los céntimos sobrantes se reparten en orden alfabético de username.
- No tiene estado: es una librería pura de funciones matemáticas.

**Pruebas automatizadas (18 pruebas):**
- **12 pruebas unitarias:** ejemplos concretos de la especificación (secciones 2.4 y 2.5), casos con residuo, un solo participante, montos grandes.
- **6 pruebas basadas en propiedades (`proptest`):** generan montos aleatorios (0.01 a 99,999.99) y cantidades aleatorias de participantes (1 a 20), verificando que INV-07 (suma de partes = total) e INV-08 (suma de balances = 0) se cumplan para **cualquier** combinación posible.

El servidor FastAPI consume esta lógica (o la reimplementa en Python con `Decimal`) para validar y calcular la división en cada registro de gasto.

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| Implementar la lógica directamente en el servidor (Python) | No permite pruebas unitarias con repositorio en memoria (la rúbrica lo exige). Además, duplicaría la lógica si el cliente también necesita calcular la vista previa. |
| Implementar la lógica en el cliente (Tauri) y enviar solo el resultado al servidor | Viola RNF-07: el servidor debe validar y calcular la división, no confiar en el cliente. |
| Usar `f64` en lugar de `rust_decimal` | Viola RNF-08 (prohibido coma flotante binaria) y causa errores de redondeo inaceptables en dinero. |
| No hacer pruebas basadas en propiedades | Viola explícitamente la rúbrica de la Entrega 6: *"pruebas basadas en propiedades que comprueban que las partes suman el total y los balances suman cero"*. |
| Mezclar las pruebas unitarias y property-based en un solo archivo | Dificulta la mantenibilidad y la defensa. Separarlas mejora la claridad conceptual. |

## Consecuencias

### Positivas
- **INV-07 garantizado:** la suma de las partes es exactamente el monto total, verificado con pruebas unitarias y property-based.
- **INV-08 garantizado:** la suma de balances es exactamente cero, verificado con pruebas unitarias y property-based.
- **RNF-08 cumplido:** toda la aritmética usa `rust_decimal`, sin coma flotante binaria.
- **Regla de residuo determinista:** los céntimos sobrantes se reparten en orden alfabético, siempre el mismo resultado con los mismos datos.
- **Cobertura exhaustiva:** 18 pruebas automatizadas cubren ejemplos concretos, casos extremos y propiedades universales.
- **Reutilizable:** el crate puede ser consumido por el servidor (Python) o por el cliente (Rust) para calcular vistas previas.
- **Cumplimiento explícito de la rúbrica:** satisface el criterio de *"pruebas automatizadas de la división... pruebas basadas en propiedades"*.

### Negativas / Restricciones
- Requiere mantener la lógica en dos lugares si el servidor la reimplementa en Python (mitigación: el servidor puede reimplementarla con `Decimal` de Python siguiendo la misma especificación).
- Las pruebas basadas en propiedades son más lentas que las unitarias (mitigación: `proptest` está configurado para ejecutar 100-256 combinaciones por prueba, no millones).

## Verificación (QA / CI)
- [ ] Las pruebas unitarias reproducen exactamente los ejemplos de las secciones 2.4 y 2.5 de la especificación.
- [ ] Las pruebas basadas en propiedades verifican INV-07 para montos aleatorios (0.01 a 99,999.99) y 1-20 participantes.
- [ ] Las pruebas basadas en propiedades verifican INV-08 para múltiples gastos aleatorios.
- [ ] El CI ejecuta `cargo test -p division` en cada push.
- [ ] El README explica cómo ejecutar las pruebas y cómo reproducir los ejemplos de la especificación.

## Referencias
- Especificación Técnica Entrega 6: Secciones 2.4 (Suma Exacta y Residuo del Redondeo) y 2.5 (Balance del Grupo).
- Criterios de Aceptación: *"pruebas automatizadas de la división... pruebas basadas en propiedades"*.
- INV-07: La suma de las partes es exactamente el monto total.
- INV-08: La suma de balances es exactamente cero.
- RNF-08: Aritmética decimal exacta, prohibido coma flotante binaria.
- ADR-006: Uso de `rust_decimal` para Manejo de Dinero.
- ADR-011: Cálculo de Saldo con Aritmética Decimal Exacta.
