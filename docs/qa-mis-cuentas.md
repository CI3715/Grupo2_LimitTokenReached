# Vista Mis cuentas — Entrega 2

Esta parte del proyecto cubre únicamente la interfaz de Mis cuentas: navegación, listado, resumen por moneda, estado vacío y formulario. El diseño de Conexión se conserva como una sección visual deshabilitada, sin comprobaciones.

Los datos de muestra están en `app/src/features/accounts/mock-data.ts`. El formulario añade cuentas al estado de React para revisar la presentación. Recargar restablece los ejemplos; no hay persistencia, llamadas Tauri, peticiones HTTP ni validaciones de negocio. Los atributos HTML del formulario ofrecen comprobaciones básicas de entrada.

No se modifica `servidor/` ni `app/src-tauri/`. Los tipos y el formato decimal son auxiliares de presentación, no una implementación del dominio.

Desde `app/`, ejecutar `npm.cmd run dev`. En el worktree con dependencias compartidas mediante junction, usar `npm.cmd run dev -- --webpack` y `npm.cmd run build -- --webpack`.

Revisión: comprobar las tres cuentas de muestra, subtotales separados, formulario, Escape y retorno del foco, estado vacío, alineación de Moneda y formato móvil sin desbordamiento. Conexión debe permanecer deshabilitada.
