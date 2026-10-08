# ADR-001: Stack Tecnológico del Cliente y del Servidor

- **Fecha:** 2026-10-06
- **Estado:** Aceptado
- **Decisores:** Grupo 2, Secc. 1 – LimitTokenReached 
- **Etiquetas:** arquitectura, stack, local-first, hexagonal

## Contexto

**Cuentas Claras** es una aplicación de finanzas compartidas con dos componentes:
1. **Cliente (escritorio + Android):** debe ser _local-first_, funcional sin conexión y con los datos del usuario almacenados prioritariamente en su dispositivo.
2. **Servidor backend:** cubre únicamente las funciones sociales (como las amistades, grupos, gastos compartidos).

El stack debe satisfacer tres restricciones no negociables:
- **Privacidad:** los datos sensibles no deben salir del dispositivo salvo lo estrictamente necesario para las funciones sociales.
- **Arquitectura hexagonal:** el dominio debe ser independiente de frameworks, UI y persistencia.
- **Costo:** el despliegue debe caber en niveles gratuitos.

## Decisión

| Capa             | Tecnología                          | Rol                                                        |
| ---------------- | ----------------------------------- | ---------------------------------------------------------- |
| Cliente – núcleo | **Tauri 2 (Rust)**                  | Shell nativo, acceso al SO, empaquetado ligero. Organizado en workspace de Cargo (crates: dominio, aplicacion, infraestructura, app, contrato). |
| Cliente – UI     | **Next.js** con `output: 'export'`  | Interfaz web estática reutilizada en escritorio y Android. |
| Servidor         | **Python + FastAPI**                | API HTTP para funciones sociales.                          |

## Alternativas consideradas

| Alternativa                  | Motivo de descarte                                                                 |
| ---------------------------- | ---------------------------------------------------------------------------------- |
| **Electron + React**         | Alto consumo de RAM y tamaño del instalador, incompatible con el objetivo local-first ligero. |
| **Flutter / Kotlin nativo**  | Duplicaría el esfuerzo de UI; se prioriza una base web única sobre Tauri.          |
| **Rust + Axum (servidor)**   | Curva de aprendizaje de `async` en Rust demasiado alta para todo el equipo en el backend. |
| **TypeScript + Hono**        | Competitivo, pero FastAPI ofrece generación nativa de OpenAPI y mayor velocidad de desarrollo en Python. |

## Consecuencias

### Positivas
- Instalador reducido y bajo consumo en comparación con Electron.
- Un solo código de UI para escritorio y Android.
- Contrato de API autogenerado y verificable desde el primer día.

### Negativas / Restricciones
- **Cliente (UI):** Next.js queda limitado estrictamente a `output: 'export'`. **Prohibido** usar Server Components, Server Actions y API Routes. El estado debe ser local o, como máximo, un contexto por pantalla.
- **Cliente (Núcleo):** Se debe usar `rusqlite` (síncrono) para almacenamiento local y `rust_decimal` para cálculos monetarios (prohibida la coma flotante binaria).
- **Servidor:** El tipado gradual de Python se compensa con `mypy --strict` obligatorio y `import-linter` para validar la hexagonalidad.
- **Contrato:** El `openapi.yaml` generado por FastAPI es la **fuente de verdad** para las pruebas de contrato.

### Verificación (QA / CI)
- [ ] La CI debe fallar si se detecta un componente prohibido de Next.js o violación de la regla de dependencia (`eslint-plugin-boundaries` / `import-linter`).
- [ ] La CI debe fallar si `mypy --strict` reporta errores.
- [ ] Las pruebas de contrato se ejecutan automáticamente contra el `openapi.yaml` versionado.

## Referencias
- [Guía Técnica del Proyecto Cuentas Claras v1.0](./)
- [Documentación de Tauri 2](https://v2.tauri.app/)
- [Next.js – Static Exports](https://nextjs.org/docs/app/building-your-application/deploying/static-exports)
- [FastAPI](https://fastapi.tiangolo.com/)
