# ADR-009: Pruebas de Dominio con Repositorio en Memoria

**Fecha:** 2026-10-13  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** pruebas, QA, repositorio-fake, memoria, TDD, INV-13

## Contexto
Los criterios de aceptación de la Entrega 2 exigen explícitamente lo siguiente:
> *"Hay pruebas automatizadas para INV-01, INV-03 e INV-18 y para cada validación de CrearCuenta, **ejecutadas con el repositorio en memoria** (sin base de datos ni red)."*

Esto busca garantizar que las reglas de negocio se puedan validar de forma aislada, rápida y sin depender de la infraestructura de persistencia (SQLite) ni de adaptadores de red.

## Decisión
Se implementará un **`RepositorioEventos` en memoria (Fake)** que almacene los eventos en un `Vec<CuentaCreada>` protegido por un `Arc<Mutex<...>>` para soportar pruebas asíncronas. 

Este repositorio en memoria implementará el mismo trait (puerto) `RepositorioEventos` que la implementación real de SQLite. Las pruebas de invariantes y validaciones de `CrearCuenta` se ejecutarán exclusivamente contra esta implementación en memoria.

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| Usar SQLite en memoria (`:memory:`) | Acopla las pruebas de dominio a la infraestructura de BD. Más lento y viola la restricción de "sin base de datos". |
| Mocking con librerías (`mockall`) | Mayor complejidad y verbosidad. Un Fake simple es más legible y fácil de mantener. |
| Pruebas de integración con SQLite real | Lentas, no aisladas, y requieren limpiar el archivo de BD entre pruebas. |

## Consecuencias

### Positivas
- **Velocidad:** las pruebas se ejecutan en milisegundos.
- **Aislamiento total:** cada prueba puede iniciar con un estado limpio sin afectar el sistema de archivos.
- **Cumplimiento de la rúbrica:** satisface directamente el criterio de aceptación de pruebas en memoria.
- **TDD:** permite al equipo escribir y validar la lógica de dominio antes de implementar el adaptador de SQLite.

### Negativas / Restricciones
- No valida el comportamiento real de SQLite (como las restricciones de llaves foráneas o transacciones complejas).
- Requiere mantener dos implementaciones del repositorio (Real y Fake).

## Verificación (QA / CI)
- [x] El comando `cargo test` ejecuta las pruebas de INV-01, INV-03 e INV-18 exitosamente.
- [x] Las pruebas de dominio no importan ni inicializan `rusqlite` ni realizan llamadas de red.
- [x] El README del proyecto explica cómo ejecutar estas pruebas.

## Referencias
- Especificación Técnica Entrega 2: Criterios de Aceptación (Pruebas automatizadas).
- Martin Fowler: Test Doubles (Fake vs. Mock).
- ADR-004: Arquitectura Hexagonal (Puertos y Adaptadores).
