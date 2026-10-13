# Vista Mis cuentas — Entrega 2

La interfaz abre directamente en Mis cuentas, sin login. Incluye listado, subtotales independientes por moneda, estado vacío, formulario y estados de carga/error. La vista de conexión en `/conexion/` comparte el diseño y la navegación de Mis cuentas.

Desde este worktree ejecuta `cd app`, `npm.cmd ci` y `npm.cmd run dev`.

En navegador se usa un adaptador de muestra con efectivo VES, banco USD y billetera USDT. El aviso explica que lo creado se pierde al recargar. No se usa localStorage ni se presenta la muestra como SQLite.

En Tauri se invocan los comandos locales. **La rama todavía no contiene esos comandos Rust ni SQLite**; hasta integrar esa parte, aparece un error de carga con Reintentar. No se modifica el backend y ninguna operación sobre cuentas usa HTTP.

## Contrato propuesto para integrar Rust

La especificación fija los nombres de comandos. Los campos y argumentos siguientes son una propuesta que debe coordinarse con quien implemente Rust.

| Comando | Argumentos | Respuesta |
| --- | --- | --- |
| `listar_cuentas` | Ninguno | `Account[]` |
| `listar_monedas` | Ninguno | `Currency[]` |
| `crear_cuenta` | `{ datos: { nombre, tipo, moneda, saldo_inicial } }` | `Account` creado |

```typescript
type Account = {
  id: string;
  nombre: string;
  tipo: "efectivo" | "banco" | "billetera";
  moneda: "VES" | "USD" | "EUR" | "USDT";
  saldo: string; // "1250.50", nunca un float JSON
};
type Currency = {
  codigo: "VES" | "USD" | "EUR" | "USDT";
  nombre: string;
  simbolo: string;
  decimales: number; // 2 en esta entrega
};
type AccountError = {
  campo?: "nombre" | "tipo" | "moneda" | "saldo_inicial";
  mensaje: string; // Para el usuario, sin trazas técnicas
};
```

El saldo se envía como texto: la coma escrita se normaliza a punto. Rust debe validar nombre, duplicados, tipo, moneda y saldo con `rust_decimal`, registrar `CuentaCreada` y persistir en SQLite. El adaptador de muestra simula respuestas para revisar el formulario; no sustituye las validaciones del dominio.

Los subtotales de presentación usan `bigint` en centavos. No se convierten los montos a `Number`, no se mezclan monedas y no se ofrece editar su moneda.

## Verificación de la interfaz

Lint y exportación estática verificados. En este worktree se usó `npm.cmd run build -- --webpack`, porque Turbopack rechaza el junction que comparte `node_modules` fuera de su raíz. Instalando dependencias propias mediante `npm ci` se puede usar el comando habitual.

Se probaron en Chrome: las tres cuentas de muestra, subtotales, nombre vacío, nombre repetido sin distinguir mayúsculas, saldo negativo, exceso de decimales, creación con coma decimal, Escape, retorno del foco, reinicio al recargar y ausencia de desbordamiento a 390px. Capturas en `docs/preview/`.

Con IPC simulado se verificaron el estado vacío, las tres invocaciones, el saldo enviado como texto, la creación, la ausencia de solicitudes externas, el error de carga sin trazas y la navegación a Conexión. Esta prueba verifica la interfaz y el contrato propuesto; no ejecuta comandos Rust reales ni SQLite.

Para la integración final verificar: estado vacío, errores por campo sin perder datos, bloqueo al guardar, creación sin HTTP, autonomía sin servidor/red y persistencia al reabrir. Las dos últimas requieren el módulo Rust. Las pruebas de invariantes y la ubicación real de SQLite corresponden a esa integración.

## Conexión

Se reemplazaron la pantalla antigua y sus estilos globales por una vista simple con la misma barra lateral, colores y tipografía que Mis cuentas. No hay canvas, partículas, orbes ni respuesta exitosa simulada.

En navegador la comprobación explica que requiere la app de escritorio. En Tauri llama a `ping_servidor` y muestra carga, éxito o un error sin trazas. El resultado esperado contiene `estado`, `mensaje`, `version`, `timestamp` y `latencia_ms`. Esta rama aún no registra el comando Rust; su integración sigue pendiente.

Se verificaron en Chrome navegación, enlace activo, formato móvil, regreso a Mis cuentas y los estados de carga, éxito y respuesta inválida con IPC simulado. Capturas: `conexion-desktop.png` y `conexion-mobile.png` en `docs/preview/`.
