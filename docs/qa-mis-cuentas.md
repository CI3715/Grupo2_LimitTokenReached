# Vista Mis cuentas — Entrega 2

Esta parte del proyecto cubre la interfaz de Mis cuentas: navegación, listado, resumen por moneda, estado vacío y formulario. La aplicación abre en la pantalla de conexión original de la Entrega 1 (`/`), con sus componentes y estilos previos al PR de Mis cuentas. El botón **Mis cuentas** abre `/mis-cuentas/`. La ruta `/conexion/` muestra la misma pantalla de conexión para conservar los enlaces existentes.

Los datos de muestra están en `app/src/features/accounts/mock-data.ts`. El formulario añade cuentas al estado de React para revisar la presentación. Recargar restablece los ejemplos; las cuentas no tienen persistencia, llamadas Tauri, peticiones HTTP ni validaciones de negocio. Los atributos HTML del formulario ofrecen comprobaciones básicas de entrada. Los tipos y el formato decimal son auxiliares de presentación, no una implementación del dominio.

## Integración posterior

- Sustituir el estado de muestra de `useAccounts` por los comandos locales `listar_cuentas`, `listar_monedas` y `crear_cuenta`, manteniendo los componentes de presentación.
- Adaptar los DTO a `Account`, `Currency` y `CreateAccountInput` de `app/src/types/accounts.ts`; los montos se representan como strings decimales, sin convertirlos a coma flotante.
- Resolver en Rust las validaciones de dominio, los eventos CuentaCreada y la persistencia SQLite. Presentar los estados de carga y errores por campo en el frontend cuando esos contratos estén disponibles.
- Mantener las cuentas independientes del servidor y del ping; crear y listar cuentas no requiere registro ni conexión.

El merge conserva `servidor/`, `app/crates/` y `app/src-tauri/` de `main`. La pantalla de conexión utiliza los componentes originales `ConnectionButton`, `InfoCard`, `LatencyBadge` y `StatusIndicator`, además de `usePing` y `pingService` existentes para invocar `ping_servidor` únicamente al pulsar el botón. En el navegador sin Tauri no puede completar esa comprobación.

## Comprobaciones

Desde `app/`, ejecutar `npm install`, `npm run lint` y `npm run build`. Para revisar la interfaz: `npm run dev`; para comprobar la conexión real: `npm run tauri dev`, con pnpm, Rust, dependencias nativas y el servidor disponibles.

Revisión manual: comprobar las tres cuentas de muestra, subtotales separados, formulario, Escape y retorno del foco, estado vacío, alineación de Moneda y formato móvil sin desbordamiento. Comprobar navegación entre Mis cuentas y Conexión. Dentro de Tauri, comprobar carga, éxito, mensaje, versión, fecha, latencia y error del ping sin afectar las cuentas.

Las imágenes en `docs/preview/` son referencias de la vista previa anterior al merge; no constituyen evidencia de pruebas de esta integración.

## Validación del merge previo

- `npm run lint`: pasó.
- `npm run build -- --webpack`: pasó, incluyendo TypeScript y exportación de `/` y `/conexion`.
- Revisión del HTML exportado: contiene Mis cuentas, Nueva cuenta, monedas de muestra y el botón de conexión.
- `git diff origin/main --check`: pasó; componentes, tipos, estado y estilos de Mis cuentas permanecen idénticos a los de la rama antes del merge.

La compilación se ejecutó con Webpack porque este entorno restringe los puertos internos de Turbopack. La revisión visual y la comprobación real dentro de Tauri quedan pendientes.

Observación heredada de `main`: las pruebas Python todavía importan `cuentas_claras.app.main`, aunque el punto de entrada fue movido a `servidor/src/main.py`. No se ejecutaron pruebas de backend ni se modificó esa capa en esta integración.

## Restauración de la pantalla de conexión

La referencia histórica es `c5b0e4a`, el primer padre del merge del PR #27 (`2794857`). Los componentes de conexión no habían sido eliminados: el PR cambió la página que los utilizaba y sus estilos globales.

Se restauraron el layout y los estilos globales de esa referencia. La página inicial recupera la pantalla original y añade el enlace a Mis cuentas; el encabezado admite que el nuevo botón pase a otra línea en pantallas pequeñas. `/conexion/` reutiliza esa misma página. Mis cuentas se trasladó a `/mis-cuentas/` con el mismo contenido y estilos; solo cambian el import relativo de CSS y el enlace de la barra lateral. El README permanece intacto.

Validación: `npm run lint`, `npm run build -- --webpack` y `git diff --check` pasaron. El HTML exportado confirma la pantalla de conexión y sus componentes en `/` y `/conexion`, la vista separada en `/mis-cuentas` y los enlaces entre ellas. Se verificó que los cuatro componentes de conexión, el hook, el servicio y los tipos coinciden con la referencia histórica, y que la vista de cuentas coincide con la anterior salvo el import de CSS.

Pendiente revisión visual y prueba real del ping dentro de Tauri.
