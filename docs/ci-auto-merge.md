# Merge automático después del CI

El workflow `.github/workflows/ci.yml` ejecuta los checks en todos los PR hacia `main`, incluso si solo cambian documentación u otros archivos. También vuelve a ejecutarse cuando un borrador se marca como listo para revisión.

El job `Auto-merge after CI` depende de los tres jobs existentes. Solo se ejecuta si todos terminaron correctamente, el evento es un PR, el PR no está en borrador y su rama pertenece al mismo repositorio. Los PR desde forks conservan el CI, pero su merge se gestiona manualmente.

## Configuración de GitHub

Una persona con permisos de administración debe configurar:

1. En **Settings → General → Pull Requests**, activar **Allow auto-merge** y mantener habilitado **Allow merge commits** (el workflow utiliza ese método).
2. En la protección de `main`, desde **Settings → Branches** o **Settings → Rules → Rulesets**, conservar **Require a pull request before merging** y desactivar **Require approvals** si el equipo quiere prescindir de revisión obligatoria. Guardar los cambios.
3. Mantener **Require status checks to pass before merging**, seleccionando estos checks de GitHub Actions:
   - `Backend (FastAPI) - Lint`
   - `Frontend (Next.js) - Lint`
   - `Rust - Contratos, núcleo y SQLite local`

No añadir `Auto-merge after CI` como check obligatorio: ese job solicita el merge después de los checks de validación. Si quedan otras reglas que bloquean el PR, GitHub las seguirá exigiendo; el workflow no las omite.

El job utiliza `GITHUB_TOKEN`, con permisos de escritura únicamente para ese job. No requiere un token personal ni aprobar PR automáticamente. Las políticas de la organización pueden limitar estos permisos; si GitHub responde con un error de permisos, debe revisarlo quien administra el repositorio.

## Comportamiento y comprobación remota

1. Abrir un PR no borrador desde una rama del mismo repositorio hacia `main`.
2. Comprobar que se ejecutan los tres jobs de validación.
3. Si cualquiera falla o se cancela, el job de merge queda omitido.
4. Si los tres pasan, el job ejecuta `gh pr merge --auto --merge` para ese PR. Se comprueba que la punta de la rama siga siendo el SHA del evento probado; si cambió, esa ejecución no debe integrar el nuevo contenido. El nuevo push genera otra ejecución del CI.
5. Verificar en GitHub el merge o los requisitos pendientes que muestre el PR. Un conflicto o una regla pendiente puede impedir el merge aunque las pruebas pasen.

Un push a `main` que dispare el CI no activa este job de merge. Los eventos generados con `GITHUB_TOKEN` normalmente no disparan otra ejecución por `push`: no se debe asumir que el merge del bot repetirá los tests en `main`. La validación que habilita este flujo es la del PR antes de integrarlo. No se hace push directo a `main`, no se fuerza el merge y no se elimina automáticamente la rama de trabajo.

La ejecución real necesita publicar el cambio y abrir un PR. Las comprobaciones locales del YAML y del comando no certifican los permisos ni las reglas activas en GitHub.

## Referencias

- [GitHub: merge automático de PR](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/automatically-merging-a-pull-request).
- [GitHub CLI: `gh pr merge`, `--auto` y `--match-head-commit`](https://cli.github.com/manual/gh_pr_merge).
- [GitHub: eventos generados con `GITHUB_TOKEN`](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow).
