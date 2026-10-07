# ADR-004: Arquitectura Hexagonal y Regla de Dependencia

- **Fecha:** 2026-10-06
- **Estado:** Aceptado
- **Decisores:** Grupo 2, Secc. 1 – LimitTokenReached
- **Etiquetas:** arquitectura, hexagonal, puertos-adaptadores, fitness-functions

## Contexto

Para garantizar la mantenibilidad, la testabilidad y el aislamiento del dominio de negocio (**RNF-13**), el sistema debe seguir una arquitectura donde las dependencias apunten siempre **hacia adentro**.

El dominio no debe saber que existe una base de datos, una interfaz gráfica o una red. Cualquier detalle de infraestructura debe poder reemplazarse sin tocar las reglas de negocio.

## Decisión

Se adoptará la **Arquitectura Hexagonal (Puertos y Adaptadores)**. El código se organizará en las siguientes capas:

| Capa               | Responsabilidad                                                  | Depende de            |
| ------------------ | ---------------------------------------------------------------- | --------------------- |
| **Dominio**        | Reglas de negocio e invariantes.                                 | Nada (capa más interna) |
| **Aplicación**     | Casos de uso y definición de **Puertos** (interfaces).           | Dominio               |
| **Infraestructura**| Implementaciones reales de los puertos (BD, red, reloj).         | Aplicación + Dominio  |
| **App / Adaptadores** | Puntos de entrada (Rutas FastAPI, Comandos Tauri).            | Aplicación            |
| **Contrato**       | Modelos de datos que viajan por la API (OpenAPI).                | Ninguna (DTOs puros)  |

**Regla de dependencia:** toda dependencia externa (reloj, red, BD) se usará a través de un **puerto declarado en la capa de aplicación**. El dominio nunca importa infraestructura.

## Alternativas consideradas

| Alternativa                              | Motivo de descarte                                                                 |
| ---------------------------------------- | ---------------------------------------------------------------------------------- |
| **Arquitectura en Capas Tradicional (MVC)** | Descartada porque tiende a filtrar detalles de infraestructura (como queries SQL) hacia la lógica de negocio. |
| **Clean Architecture**                   | Muy similar. Se adopta la variante **hexagonal** por su énfasis explícito en los puertos de entrada/salida. |

## Consecuencias

### Positivas
- El dominio queda **aislado** y es testeable sin base de datos, sin red y sin UI.
- Los detalles de infraestructura (FastAPI, Tauri, Postgres, etc.) son **reemplazables** sin tocar las reglas de negocio.
- Los puertos actúan como **contratos explícitos** entre capas, lo que facilita el trabajo en paralelo.

### Negativas / Restricciones
- **Más indirección:** cada acceso externo requiere definir un puerto y al menos una implementación. Para features pequeñas puede parecer sobre-ingeniería.
- **Disciplina verificable:** la regla de dependencia no puede quedar solo en la cabeza del equipo; debe automatizarse.

### Fitness Functions
Para el cumplimiento de esta regla, se implementarán pruebas automáticas que **bloquearán la integración (CI)** si el dominio o la aplicación importan infraestructura:
- **Python:** `import-linter` con contratos de capas prohibidas.
- **TypeScript/Rust:** `eslint-plugin-boundaries` o revisión de visibilidad de módulos/crates.

### Pruebas de Contrato
Cada puerto tendrá **al menos dos implementaciones**:
1. Una **real** (ej. Postgres, `reqwest`, reloj del sistema).
2. Una **Fake / In-Memory**.

Ambas pasarán el **mismo conjunto de pruebas de contrato**, garantizando que el comportamiento observable sea idéntico.

### Verificación
- [ ] CI falla si el dominio importa infraestructura.
- [ ] CI falla si la aplicación importa infraestructura directamente (solo puede importar puertos).
- [ ] Cada puerto tiene al menos una implementación real y una fake.
- [ ] Las pruebas de contrato de cada puerto se ejecutan contra ambas implementaciones.

## Referencias
- [Alistair Cockburn – Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [import-linter – Documentation](https://import-linter.readthedocs.io/)
- [eslint-plugin-boundaries](https://github.com/javierbrea/eslint-plugin-boundaries)
