# ADR-012: Eventos locales de tasas y monedas

**Fecha:** 2026-10-27  
**Estado:** Propuesto; implementación y revisión del equipo pendientes  
**Alcance:** Entrega 4, secciones 2.1, 2.2 y 2.4; RF-CAM-01/02/03/06

## Contexto

El libro local debe conservar las tasas personales, su historial, las monedas propias y la moneda base sin depender del servidor. Eliminar una tasa no puede modificar movimientos ni borrar eventos. Se extiende la decisión de almacenamiento local del [ADR-005](005-local-first-event-sourcing.md) y la inmutabilidad del [ADR-010](010-event-sourcing-movimientos.md).

## Decisión propuesta

Registrar `TasaRegistrada`, `TasaEliminada`, `MonedaBaseCambiada` y `MonedaCreada` mediante inserciones en el libro SQLite. `TasaEliminada` referencia la tasa original y la excluye de las conversiones sin borrar su historial. Las proyecciones deben poder reconstruirse desde los eventos; nunca ejecutar `UPDATE` o `DELETE` sobre la tabla de eventos.

La validación corresponde al dominio, con errores de campo: monedas existentes y distintas, valor positivo con hasta ocho decimales, fecha válida no futura y fuente no vacía. El reloj entra por un puerto para probar fechas sin depender del día de ejecución. Valor y montos cruzan la frontera JSON como texto decimal.

La moneda base comienza en USD y debe pertenecer al catálogo. Cambiarla solo afecta la presentación del patrimonio. Crear una moneda exige código único de tres a cinco letras mayúsculas, nombre, símbolo y entre cero y ocho decimales. VES, USD, EUR y USDT son monedas de fábrica protegidas.

Las tasas y preferencias permanecen en el dispositivo; estas operaciones no invocan FastAPI ni proveedores externos. Tauri debe delegar en los casos de uso y estos en los puertos de persistencia.

## Alternativas

- Borrar físicamente tasas: pierde trazabilidad y contradice la sección 2.4.
- Guardar una conversión en cada movimiento: alteraría el significado del libro y dificultaría cambiar de base.
- Persistir solo en el navegador o servidor: no satisface la persistencia local del libro requerida por la entrega.

## Consecuencias

Se conserva un historial auditable y las tasas erróneas pueden dejar de aplicarse. Se requieren proyecciones del catálogo, base y tasas activas, además de pruebas de reconstrucción y persistencia. El orden de registro debe ser estable para resolver tasas del mismo par y fecha; no basta ordenar solo por fecha civil.

## Verificación pendiente

- [ ] Dominio rechaza cada entrada inválida sin agregar eventos.
- [ ] Cada operación genera el evento correspondiente y sobrevive al reinicio.
- [ ] Eliminar una tasa conserva el evento original y todos los movimientos.
- [ ] Cambiar base conserva saldos y movimientos; crear moneda permite usarla en una cuenta.
- [ ] Red deshabilitada y servidor apagado: operaciones y reinicio conservan resultados.

## Estado observado

Existen DTOs en `app/crates/contrato/src/{tasa,moneda}.rs`. No están implementados los eventos, casos de uso ni comandos Tauri de esta decisión. Las casillas indican criterios de aceptación, no resultados obtenidos.
