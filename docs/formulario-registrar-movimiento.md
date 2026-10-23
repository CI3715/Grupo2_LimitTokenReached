# Formulario de registro de movimientos — Rosa

Implementación frontend basada en el contrato del PR #40 y la especificación de Entrega 3, secciones 2.1 y 2.5. No modifica el dominio, los comandos Rust, la persistencia, las categorías ni la vista de detalle.

## Integración con el detalle de Carlos

Importar `RegisterMovement` desde `@/features/movements/register-movement` en la vista de detalle. Montarlo cuando se abra el formulario y desmontarlo al cerrar. Recibe:

| Propiedad | Contenido |
| --- | --- |
| `account` | `id`, `nombre`, `moneda_codigo`, `saldo` actual como string |
| `currency` | Moneda de esa cuenta, con símbolo y decimales reales |
| `categories` | `CategoriaDTO[]` del catálogo real: `id`, `nombre`, `tipo` |
| `categoriesLoading` | Opcional: carga pendiente del catálogo |
| `categoriesError` | Opcional: mensaje si falla la carga |
| `onRetryCategories` | Opcional: volver a cargar el catálogo |
| `onRegistered(movement)` | Refrescar saldos, composición y movimientos desde el backend |
| `onCancel()` | Cerrar el formulario |

Ejemplo de composición dentro de la vista existente de detalle (las variables las aporta esa vista):

```tsx
{formOpen && (
  <RegisterMovement
    account={account}
    currency={currency}
    categories={categories}
    categoriesLoading={categoriesLoading}
    categoriesError={categoriesError}
    onRetryCategories={reloadCategories}
    onCancel={() => setFormOpen(false)}
    onRegistered={async () => {
      await refreshAccountDetail();
      await refreshAccounts();
    }}
  />
)}
```

Se muestra confirmación de éxito y se bloquea otro envío hasta cerrar. Si falla el refresco después de guardar, se informa que el movimiento ya se guardó; no se vuelve a invocar el registro. Cambiar de cuenta reinicia el formulario. Si el contenedor usa un diálogo, debe impedir su cierre durante el envío.

## Acuerdo pendiente con Víctor

El servicio realiza la llamada real:

```ts
invoke("registrar_movimiento", {
  datos: { cuenta_id, tipo, monto, fecha, categoria_id, nota },
});
```

`datos` sigue la convención actual de `crear_cuenta`. Confirmar ese nombre con Víctor cuando publique el comando. La respuesta esperada es `Result<MovimientoDTO, ErrorValidacionDTO>`: Tauri resuelve el DTO o rechaza con `{ campo, mensaje }`. También se admiten errores serializados como JSON y mensajes de texto. El frontend no presupone un contenedor `{ ok, data }`.

El servicio verifica la forma de la respuesta y su correspondencia con el registro solicitado; admite diferencias de ceros decimales (`50.00` y `50`). Una respuesta inesperada bloquea el reenvío y solicita revisar la cuenta, porque el registro podría haberse guardado.

## Comportamiento y alcance

- Tipo: exclusivamente `Ingreso` y `Gasto`. No existe opción de ajuste ni corrección.
- Monto: texto con punto o coma decimal, normalizado a punto sin convertir a `number`. Se muestra la precisión de la moneda; el dominio decide las reglas de monto y precisión.
- Fecha: hoy según la zona local del dispositivo; ayuda de interfaz con máximo hoy.
- Categoría: obligatoria, usa el ID real y se limpia al cambiar de tipo. Sin catálogo, no se puede enviar; no se generan categorías ficticias.
- Nota: texto opcional; vacía o espacios se envía como `null`.
- Saldo negativo: se compara el gasto con el saldo mediante enteros `BigInt` escalados a la precisión de los textos. Se advierte y pide confirmar antes del único `invoke`. No se calcula ni persiste un saldo frontend.
- Errores de dominio: mensaje general y junto al campo correspondiente; se conservan los valores para corregirlos. `categoria`, `categoría` y `categoria_id` se muestran en el selector de categoría.

## Verificación

Desde `app`:

```sh
npm run test:movements
npm run lint
npx tsc --noEmit
```

Las pruebas usan Node, React y TypeScript ya instalados. Verifican el payload y nombre del comando con un `invoke` simulado, respuestas incompatibles, errores de dominio, comparación decimal exacta y render inicial. No sustituyen las pruebas del dominio ni una prueba Tauri real.

Pendiente de integración: publicar y registrar el comando de Víctor; proporcionar el catálogo real y montar el formulario desde el detalle de Carlos. Después comprobar en Tauri un ingreso, un gasto, saldo negativo con cancelar/confirmar, errores de cada campo, doble clic y actualización del saldo/listado. El formulario no está conectado a `page.tsx`, para no implementar una vista de detalle ajena al alcance de Rosa.
