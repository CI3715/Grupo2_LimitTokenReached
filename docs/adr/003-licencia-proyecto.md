# ADR-003: Licencia de Código Abierto

- **Fecha:** 2026-10-06
- **Estado:** Aceptado
- **Decisores:** Grupo 2, Secc. 1 – LimitTokenReached
- **Etiquetas:** licencia, open source, legal, Apache-2.0

## Contexto

La Guía Técnica del proyecto **Cuentas Claras** exige que:
1. Todos los repositorios sean **públicos desde el primer día**.
2. Todas las piezas compartan la **misma licencia**.

Sin un archivo `LICENSE`, un repositorio público queda por defecto como **"todos los derechos reservados"**, lo que impide legalmente que otros usen, modifiquen o contribuyan al código, incluso aunque sea visible.

Además, el proyecto tiene dos componentes con dependencias de terceros (app cliente y servidor), por lo que la licencia elegida debe ser **compatible con el ecosistema** y proteger tanto a los contribuyentes como a los usuarios.

## Decisión

Se utilizará la licencia **Apache License 2.0** para todas las piezas del sistema:
- `/app` (cliente)
- `/servidor` (backend)
- `/docs` (documentación)

## Alternativas consideradas

| Alternativa       | Motivo de descarte                                                                 |
| ----------------- | ---------------------------------------------------------------------------------- |
| **MIT**           | Más simple, pero **no incluye una concesión expresa de patentes**. Para un proyecto que puede recibir contribuciones de terceros, la protección de patentes de Apache-2.0 es preferible. |
| **GPL / AGPL**    | Copyleft fuerte. **AGPL** obligaría a liberar el código de cualquier servicio en red que use este proyecto, lo cual es demasiado restrictivo para un proyecto académico que busca adopción. |
| **BSD-3-Clause**  | Similar a MIT, sin concesión expresa de patentes. Mismas razones que MIT.          |

## Consecuencias

### Positivas
- **Permisiva:** permite construir productos cerrados encima del proyecto, lo que favorece la adopción.
- **Concesión de patentes:** protege a los contribuyentes frente a reclamaciones de patentes por parte de otros usuarios del código.
- **Compatible con GPL-3.0:** el código Apache-2.0 puede incluirse en proyectos GPL-3.0 (no al revés).
- **Estándar reconocido:** GitHub detecta y muestra la licencia automáticamente si el archivo se llama `LICENSE` en la raíz.

### Negativas / Restricciones
- **Incompatibilidad con GPL-2.0:** Apache-2.0 **no es compatible con GPL-2.0** debido a sus cláusulas de patentes. Si una dependencia usa GPL-2.0, no se puede incluir directamente en el proyecto.
- **Obligación de NOTICE:** si se incluyen dependencias que traen su propio archivo `NOTICE`, sus atribuciones deben propagarse al `NOTICE` del proyecto. Esto no aplica al código propio, sino al de terceros.
- **Mantenimiento:** se debe mantener un archivo `THIRD-PARTY-NOTICES` actualizado para garantizar la compatibilidad de todas las dependencias con Apache-2.0.

### Verificación
- [ ] Existe un archivo `LICENSE` en la raíz del monorepo.
- [ ] La CI incluye una verificación de compatibilidad de licencias de dependencias (por ejemplo, con `license-checker`, `pip-licenses` o similar).
- [ ] El archivo `NOTICE` (si existe) propaga las atribuciones requeridas por dependencias Apache-2.0 que incluyan su propio `NOTICE`.
- [ ] No se incluyen dependencias con licencias incompatibles (GPL-2.0, AGPL si el proyecto no es AGPL).

## Referencias
- [Apache License 2.0 – Texto oficial](https://www.apache.org/licenses/LICENSE-2.0)
- [Apache License v2.0 and GPL Compatibility](https://www.apache.org/licenses/GPL-compatibility.html)
- [GitHub – Adding a license to a repository](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/adding-a-license-to-a-repository)
- [Assembling LICENSE and NOTICE files (Apache)](https://infra.apache.org/licensing-howto.html)
