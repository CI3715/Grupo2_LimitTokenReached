# Especificación del Formato de Exportación (Cuentas Claras)

El formato plano del libro es texto UTF-8 con tres líneas de cabecera y un evento por línea. Puede procesarse fuera de la aplicación (RNF-04). La envoltura del respaldo cifrado se describe por separado en el [ADR-019](adr/019-cifrado-respaldo.md).

## 1. Cabecera

Las primeras tres líneas aparecen en este orden:

```text
VERSION=1.0
FECHA_EXPORTACION=2026-11-25T10:00:00Z
TOTAL_EVENTOS=2
```

`VERSION` identifica la versión del libro plano. `FECHA_EXPORTACION` contiene la fecha de exportación y `TOTAL_EVENTOS` el número de líneas de eventos. La implementación actual admite la versión `1.0` y comprueba que la cantidad coincida con el cuerpo.

## 2. Línea de evento

```text
TIPO_EVENTO|ID_EVENTO|TIMESTAMP|DEVICE_ID|DATOS_JSON
```

Los primeros cuatro campos no pueden estar vacíos ni contener `|` o saltos de línea al exportar. Separar solo los primeros cuatro `|`: el JSON puede contener ese carácter. `TIMESTAMP` representa la fecha del evento; los eventos de la aplicación usan marcas RFC 3339. `DEVICE_ID` identifica el origen y puede ser `servidor`. `ID_EVENTO` identifica el evento inmutable: los eventos locales usan UUID v4 y los eventos derivados de gastos grupales usan UUID v5 determinísticos; no exigir exclusivamente v4. Los datos son JSON serializado en una sola línea. El serializador filtra claves de credenciales y tokens y enmascara cadenas con forma de JWT.

Ejemplo de una exportación con dos eventos (cada evento ocupa una línea):

```text
VERSION=1.0
FECHA_EXPORTACION=2026-11-25T10:00:00Z
TOTAL_EVENTOS=2
CuentaCreada|00000000-0000-4000-8000-000000000001|2026-11-25T10:00:00Z|dispositivo-1|{"id":"cuenta-1","nombre":"Efectivo","tipo":"Efectivo","moneda_codigo":"USD","saldo_inicial":"100.00","creada_en":"2026-11-25"}
MovimientoRegistrado|00000000-0000-4000-8000-000000000002|2026-11-25T10:05:00Z|dispositivo-1|{"id":"movimiento-1","cuenta_id":"cuenta-1","tipo":"Gasto","monto":"20.00","fecha":"2026-11-25","categoria_id":"categoria-1","nota":null}
```

## 3. Tipos de eventos y datos

Los nombres siguientes corresponden a los tipos que la proyección SQLite reconoce. La columna de datos muestra las claves JSON relevantes; los montos y valores decimales se representan como cadenas para mantener la precisión. Los campos adicionales pueden existir en eventos históricos.

| `TIPO_EVENTO`          | `DATOS_JSON` y comportamiento                                                                                                                                                                                                                                                           |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CuentaCreada`         | `id`, `nombre`, `tipo`, `moneda_codigo`, `saldo_inicial`, `creada_en`. Crea la cuenta y fija su saldo inicial. La proyección admite valores predeterminados para algunos campos históricos.                                                                                             |
| `CuentaRenombrada`     | `id`, `nombre`. Actualiza el nombre sin cambiar el saldo.                                                                                                                                                                                                                               |
| `CuentaArchivada`      | `id`. Marca la cuenta inactiva sin modificar su saldo.                                                                                                                                                                                                                                  |
| `MovimientoRegistrado` | `id`, `cuenta_id`, `tipo` (`Ingreso` o `Gasto`), `monto`, `fecha`, `categoria_id`, `nota` (puede ser `null`). Un ingreso suma el monto; un gasto lo resta.                                                                                                                              |
| `MovimientoCorregido`  | `id`, `movimiento_original_id`, `cuenta_id`, `tipo_original`, `variacion_monto`; `fecha` es opcional. La variación se suma para ingresos y se resta para gastos; puede ser negativa. Para históricos, la proyección admite `movimiento_id` como alternativa a `movimiento_original_id`. |
| `GastoDeGrupoAplicado` | `id_evento`, `movimiento_id`, `grupo_id`, `gasto_id`. Registra la aplicación del gasto grupal; el efecto en dinero proviene del `MovimientoRegistrado` asociado, no de esta marca.                                                                                                      |
| `MonedaCreada`         | `codigo`, `nombre`, `simbolo`, `decimales` (número entero); puede incluir `es_del_catalogo`. Actualiza el catálogo de monedas.                                                                                                                                                          |
| `MonedaEliminada`      | `codigo`. Elimina la moneda de la proyección del catálogo.                                                                                                                                                                                                                              |
| `CategoriaCreada`      | `id`, `nombre`, `tipo`. Actualiza el catálogo de categorías.                                                                                                                                                                                                                            |
| `TasaRegistrada`       | `tasa_id`, `moneda_origen`, `moneda_destino`, `valor`, `fecha`, `fuente`. Registra una tasa para conversiones, sin sumar dinero por sí sola al saldo de una cuenta.                                                                                                                     |
| `TasaEliminada`        | `tasa_id`. Marca la tasa como eliminada.                                                                                                                                                                                                                                                |

La proyección conserva los eventos de tipo desconocido en el historial, pero no les asigna un efecto en las proyecciones conocidas. El script independiente también acepta algunos alias históricos propios; eso no significa que la proyección SQLite traduzca esos alias.

## 4. Reconstrucción determinista de saldos (RNF-04)

1. Unir eventos por `ID_EVENTO`: ignorar duplicados idénticos y rechazar con motivo el mismo ID con distinto contenido o metadatos, sin sustituir el evento existente.
2. Ordenar por `(TIMESTAMP, DEVICE_ID, ID_EVENTO)`. La aplicación compara las cadenas almacenadas; el script independiente interpreta los timestamps como instantes UTC. Para obtener el mismo orden entre ambas implementaciones, usar marcas UTC normalizadas con precisión uniforme; formatos equivalentes con distintas zonas o precisiones deben validarse antes de asumir equivalencia.
3. Reconstruir catálogos, cuentas y movimientos, y después aplicar correcciones y marcas grupales que referencian movimientos. Por cuenta y moneda: partir del saldo inicial, sumar ingresos, restar gastos y aplicar cada variación según su tipo original. Las marcas grupales no deben duplicar su movimiento asociado.
4. Usar aritmética decimal exacta, sin `float`. Las tasas pertenecen al catálogo de conversión y no alteran por sí solas el saldo nominal de una cuenta.

La aplicación conserva el historial desconocido; una herramienta que lo ignore solo puede garantizar los saldos derivados de los tipos que entiende. El script [reconstruir_saldos.py](../scripts/reconstruir_saldos.py) ofrece una reconstrucción independiente de los tipos que declara soportar. Las diferencias de orden temporal y de compatibilidad histórica deben considerarse al comparar resultados.

## 5. Referencias

- [Serializador del libro](../app/crates/infraestructura/src/serializador_libro.rs).
- [Proyección y fusión SQLite](../app/crates/infraestructura/src/repositorio_eventos_sqlite.rs).
- [Orden de eventos](../app/crates/dominio/src/libro.rs).

## 6. Integración de los contratos

- **Backend:** implementar la serialización, el cifrado, la fusión y los comandos usando los contratos del libro. Corresponde a backend conectar los errores de negocio desde su origen y mantener los rechazos individuales de eventos en el reporte.
- **Frontend:** consumir los DTOs y códigos expuestos por los comandos, mostrar los contadores y rechazos, y presentar mensajes claros sin exponer detalles técnicos.

El catálogo `ErrorSincronizacion` y los DTOs están definidos en `app/crates/contrato/src/`. En la implementación actual, las capas internas devuelven cadenas y los comandos las clasifican en códigos para la interfaz; la unificación tipada desde su origen corresponde a la integración de backend.
