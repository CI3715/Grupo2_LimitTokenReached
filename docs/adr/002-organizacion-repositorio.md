# ADR-002: Organización del Repositorio (Monorepo)

- **Fecha:** 2026-10-06
- **Estado:** Aceptado
- **Decisores:** Grupo 2, Secc. 1 – LimitTokenReached
- **Etiquetas:** repositorio, monorepo, CI, contrato

## Contexto

El sistema **Cuentas Claras** tiene dos piezas desplegables:
1. La **aplicación cliente** (Next.js + Tauri 2).
2. El **servidor backend** (FastAPI).

Además, existe un tercer elemento que ambos comparten: el **contrato de la API** (`openapi.yaml`), que según el ADR-001 es la fuente de verdad para las pruebas de contrato.

Necesitamos decidir cómo organizar el código fuente para que:
- El equipo pueda colaborar sin problema.
- La integración continua sea simple de mantener.
- El contrato de la API se mantenga sincronizado entre cliente y servidor.

## Decisión

Se utilizará una estructura de **monorepo** en un único repositorio público, con las siguientes carpetas base:

| Carpeta     | Contenido                                              |
| ----------- | ------------------------------------------------------ |
| `/app`      | Código de la aplicación cliente (Next.js + Tauri 2).   |
| `/servidor` | Código del backend (FastAPI).                          |
| `/docs`     | Documentación (ADRs, diagramas C4, manuales).          |
| `/scripts`  | Scripts de automatización y pruebas de sistema.        |

## Alternativas consideradas

| Alternativa                          | Motivo de descarte                                                                 |
| ------------------------------------ | ---------------------------------------------------------------------------------- |
| **Repositorios separados** (`cuentas-claras-app` y `cuentas-claras-servidor`) | Ofrece mayor aislamiento, pero complica la referencia cruzada al `openapi.yaml` y obliga a mantener dos pipelines de CI y dos archivos de licencia sincronizados. |

## Consecuencias

### Positivas
- Un solo lugar para gestionar problemas, Pull Requests y la CI.
- El cliente puede consumir directamente el `openapi.yaml` del servidor para las pruebas de contrato.
- Un único archivo de licencia y un único pipeline base.

### Negativas / Restricciones
- El historial de Git mezclará cambios de frontend y backend.
  - **Mitigación:** usar convenciones estrictas de nombres de ramas (`feature/...`, `fix/...`) y, si hace falta, prefijos por área (`app/...`, `servidor/...`).
- La CI debe configurarse para ejecutar solo lo que cambió (path filters), evitando correr todo el pipeline en cada push.

### Verificación
- [ ] La CI usa filtros por ruta para no ejecutar pruebas innecesarias.
- [ ] El `openapi.yaml` vive en una sola ubicación y ambos lados lo referencian desde ahí.
- [ ] El repositorio tiene un único `LICENSE` en la raíz.

## Referencias
- [ADRs – Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [Monorepo vs. Polyrepo](https://www.atlassian.com/git/tutorials/monorepos)
