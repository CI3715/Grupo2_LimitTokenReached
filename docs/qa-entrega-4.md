# QA de Entrega 4: monedas, tasas y patrimonio

Referencia: Especificación Técnica Entrega 4, secciones 2.1–2.5 y criterios de aceptación. Esta guía describe el comportamiento esperado para evaluación y distingue las pruebas disponibles de las que faltan.

## Estado de cobertura

Revisión del 27/10/2026: están integrados los DTOs y nueve pruebas de contrato de tasas/patrimonio. No hay todavía casos de uso, repositorio de tasas, comandos Tauri ni pantallas de Entrega 4. El flujo manual queda pendiente de esa integración.

| Área                   | Evidencia disponible                                                                             | Pendiente                                                                                            |
| ---------------------- | ------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------- |
| Contratos              | Seis tests de tasas y tres de patrimonio; montos como `String`, total opcional y pares faltantes | Serialización y errores reales de los casos de uso                                                   |
| INV-02                 | Respuestas de ejemplo con `None` y `Some` construidas por el test                                | Probar que el cálculo impide consolidar si falta tasa y que no acepta tasas cruzadas                 |
| INV-03                 | Tipo `String` del valor de la tasa                                                               | Inversa, precisión de conversiones y redondeo final del total                                        |
| Validaciones           | Requests con valores inválidos que los tests pueden identificar                                  | Ejecutar dominio y comprobar rechazo con campo y mensaje; fecha ausente y precisión máxima incluidas |
| Fecha/tasa vigente     | Sin prueba de comportamiento E4                                                                  | Corte histórico, desempate, tasas eliminadas y saldos a la fecha                                     |
| Persistencia/autonomía | Regresión SQLite de movimientos, categorías y lectura del catálogo                               | Eventos E4, reinicio y prueba real sin red ni servidor                                               |

Una prueba que asigna `None` a una respuesta no demuestra que el dominio impida sumar monedas distintas. Las pruebas actuales de E4 no usan un repositorio fake. SQLite `:memory:` verifica adaptadores y tampoco sustituye el repositorio fake sin BD exigido para el núcleo E4.

## Comandos automatizados

Desde la raíz del repositorio, con Rust/Cargo instalados:

```bash
cd app
cargo test -p contrato -p aplicacion -p dominio
cargo test -p contrato tasa_tests::
cargo test -p contrato patrimonio_tests::
# Regresión de adaptadores locales; no requiere FastAPI
cargo test -p infraestructura --test repositorio_movimientos_sqlite --test repositorio_categorias_sqlite --test repositorio_monedas_sqlite
```

Los dos comandos filtrados son una alternativa para revisar solo los grupos nuevos; ya están incluidos en el primero. Para regresión frontend, desde `app/`, con las dependencias instaladas:

```bash
pnpm run test:movements
pnpm run lint
pnpm run build
```

No ejecutar `cargo test --workspace` como prueba aislada sin servidor: incluye `conexion_servidor`, que hace una petición real a FastAPI. El CI selecciona explícitamente las suites locales y ejecuta las pruebas de movimientos frontend, lint y build. Los cambios de README y `docs/` también activan el workflow. Un CI verde con las suites actuales no certifica la Entrega 4 completa.

## Ejemplo 2.3 para la evaluación

**Precondición:** integrar backend y pantallas de Entrega 4. Usar un perfil de evaluación sin otras cuentas ni tasas que alteren los totales. Conservar intacto el libro personal; no borrar su BD. El día de evaluación `D` debe ser igual o posterior al 21/10/2026. Consultar inicialmente `D`, de modo que las cuentas recién creadas existan al corte.

1. Iniciar la aplicación nativa con `cd app` y `pnpm run tauri dev`, teniendo instaladas las dependencias de frontend y las nativas de Tauri.
2. Crear estas tres cuentas. Los valores de entrada usan punto decimal, sin separador de miles:

   | Cuenta             | Moneda | Saldo inicial |
   | ------------------ | ------ | ------------- |
   | Efectivo bolívares | VES    | `88700.00`    |
   | Banco en dólares   | USD    | `200.00`      |
   | Billetera USDT     | USDT   | `50.00`       |

3. Registrar las dos tasas y comprobar su historial:

   | Origen | Destino | Valor (destino por 1 origen) | Fecha        | Fuente         |
   | ------ | ------- | ---------------------------- | ------------ | -------------- |
   | USD    | VES     | `887.00`                     | `2026-10-20` | BCV            |
   | USDT   | VES     | `1000.00`                    | `2026-10-21` | casa de cambio |

4. Elegir base VES y consultar patrimonio en `D`. Verificar saldos, subtotales y tasas usadas:

   | Cuenta             | Conversión                              | En VES         |
   | ------------------ | --------------------------------------- | -------------- |
   | Efectivo bolívares | Sin conversión                          | 88.700,00      |
   | Banco en dólares   | 200 × 887                               | 177.400,00     |
   | Billetera USDT     | 50 × 1.000                              | 50.000,00      |
   | **Total**          | Un único redondeo final a dos decimales | **316.100,00** |

5. Cambiar base a USD manteniendo `D`. Los saldos originales deben seguir siendo 88.700,00 VES, 200,00 USD y 50,00 USDT. Efectivo usa la inversa: `88700 ÷ 887 = 100.00 USD`. **No debe aparecer ningún total consolidado**, ni permanecer el anterior. Deben verse los subtotales originales y el mensaje «Falta una tasa de USDT a USD con fecha igual o anterior al [D]». No se permite usar USDT→VES→USD.
6. Registrar `USDT→USD`, valor `1.00`, fecha `2026-10-21`, fuente `estimación propia`. Consultar otra vez: `100 + 200 + 50 = 350.00 USD`. Confirmar las tasas declaradas, incluida la inversa USD→VES de fecha 20/10 y fuente BCV.
7. Eliminar la tasa USDT→USD recién creada. Debe conservarse su trazabilidad y volver el aviso de tasa faltante sin total. Registrar nuevamente la equivalencia para recuperar 350,00 USD. Comprobar que no cambió ningún movimiento.

Para mostrar literalmente el aviso del **23/10/2026**, se necesita un perfil de prueba con las tres cuentas existentes en esa fecha y consultar `2026-10-23`. No cambiar el reloj del sistema ni asumir que cuentas creadas después existían entonces. La preparación de ese perfil histórico queda pendiente del soporte de backend.

## Casos de dominio que faltan automatizar

Cada test debe crear su propio repositorio fake en memoria y usar un reloj fijo (por ejemplo, 23/10/2026). Debe invocar el código productivo, no reimplementar el cálculo en el test.

| Caso                | Preparación y resultado exigido                                                                                                                                                                 |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tasa inválida       | Sin fecha, sin fuente o con espacios, cero, negativo, fecha inválida/futura, monedas iguales o fuera del catálogo, más de ocho decimales: error del campo correspondiente, sin persistir evento |
| INV-02              | Tres cuentas del ejemplo, base USD, sin USDT→USD: total ausente y par/fecha faltantes; no resolver mediante VES                                                                                 |
| Corte de tasas      | USD→VES 800 el 19/10, 887 el 20/10 y 900 el 22/10: corte 21/10 usa 887; corte 19/10 usa 800; corte 18/10 informa tasa faltante                                                                  |
| Empate              | Dos tasas del mismo par y fecha: prevalece la última registrada, incluso tras reconstruir el repositorio                                                                                        |
| Tasa eliminada      | La tasa eliminada no se aplica; se usa la siguiente vigente o se declara el par faltante                                                                                                        |
| Inversa             | Sin VES→USD directa, saldo 88.700 VES con USD→VES 887: resultado 100 USD e indicación de inversa                                                                                                |
| Preferencia directa | Si hay directa aplicable, usarla; recurrir a inversa cuando no exista directa aplicable                                                                                                         |
| Redondeo final      | Dos cuentas de 0,01 EUR, tasa EUR→USD 0,5: contribuciones 0,005 + 0,005, total 0,01 USD; detecta redondeo por cuenta antes de sumar                                                             |
| Decimales de base   | Repetir redondeo con bases de 0 y 8 decimales; acordar y probar empates de redondeo                                                                                                             |
| Saldo histórico     | Un movimiento posterior al corte y una cuenta creada después no alteran el patrimonio previo                                                                                                    |
| Base y catálogo     | USD inicial; cambio de base conserva saldos; código propio único de 3–5 mayúsculas y 0–8 decimales; catálogo de fábrica protegido                                                               |

## Prueba manual de persistencia y autonomía

Usar el mismo perfil del ejemplo y guardar capturas de historial, base, saldos y patrimonio antes del cierre.

1. Preparar previamente un ejecutable nativo independiente del servidor de desarrollo. Desde `app/`: `pnpm run tauri build --debug --no-bundle`. Esta compilación requiere dependencias instaladas y puede necesitar red; se hace **antes** de desconectarse. Si falla, registrar el error y resolverlo antes de la prueba. No se incluye aquí un cambio de configuración nativa para forzar la compilación.
2. Abrir el ejecutable generado en `app/target/debug/` (nombre definido en `app/src-tauri/Cargo.toml`; en Windows termina en `.exe`). Confirmar que usa el mismo perfil; si no, preparar el ejemplo en ese perfil antes de continuar.
3. Cerrar completamente la app, detener FastAPI y el servidor Next.js, y desactivar Wi-Fi/Ethernet. Registrar que ambas condiciones se cumplen.
4. Abrir directamente el ejecutable. Verificar tasas, moneda base USD, catálogo, saldos, subtotales y total 350,00 USD iguales a los registrados.
5. Sin red, eliminar la tasa USDT→USD y comprobar el bloqueo del total. Cerrar y reabrir: la eliminación, el aviso y los subtotales persisten.
6. Sin red, registrar de nuevo la tasa y cambiar base a VES. Cerrar y reabrir: total 316.100,00 VES, base e historial persisten.
7. Crear una moneda propia válida y una cuenta en ella, y comprobar su persistencia al reiniciar. Si no hay tasa hacia la base, debe bloquear la consolidación e identificar el nuevo par.

`tauri dev` depende de un servidor frontend local; no demuestra por sí solo el arranque autónomo con los servidores detenidos.

## Evidencia automatizada local (27/10/2026)

- `cargo test -p contrato -p aplicacion -p dominio`: 43 pruebas aprobadas (20 de contrato, 23 de dominio; aplicación aún tiene cero tests).
- Suites SQLite seleccionadas: nueve pruebas aprobadas (cinco de movimientos, tres de categorías y una de lectura de monedas).
- Se corrigió el adaptador de monedas para completar `es_del_catalogo`, requerido por el DTO nuevo. La regresión comprueba las cuatro monedas de fábrica y una moneda propia insertada como fixture; no prueba el caso de uso de creación de monedas.
- `npm run test:movements` y `npm run lint`: aprobados tras instalar las dependencias declaradas del frontend. La ejecución directa `node scripts/movements.test.mjs` confirma 14 pruebas aprobadas.
- `node node_modules/typescript/bin/tsc --noEmit`: aprobado tras apartar la caché generada `.next/dev` con referencias a rutas antiguas.
- Compilación frontend no validada: `npm run build` falló por permisos al abrir un puerto interno de Turbopack; el intento alternativo `npm run build -- --webpack` falló al interpretar la salida de TypeScript `--showConfig`. No se cambió el compilador del CI ni se considera aprobado el build.
- YAML del workflow, enlaces locales de los documentos y `git diff --check`: correctos.

Estos resultados no incluyen la ejecución remota de GitHub Actions ni la evaluación manual de Entrega 4.

## Registro de evaluación

Completar tras integrar y ejecutar cada caso. No marcar como aprobado lo que esté bloqueado o sin ejecutar.

| Caso                            | Estado inicial                                        | Evidencia a completar                           |
| ------------------------------- | ----------------------------------------------------- | ----------------------------------------------- |
| Contratos tasas/patrimonio      | Disponible; ejecución automatizada separada del flujo | Comando, resultado y revisión evaluada          |
| Dominio E4 con fake             | Pendiente de implementación                           | Tests de selección, inversa, redondeo y bloqueo |
| Ejemplo 2.3                     | Bloqueado por integración E4                          | Capturas 316.100 VES, aviso y 350 USD           |
| Historial, base y moneda propia | Bloqueado por integración E4                          | Capturas y comparación de saldos                |
| Reinicio sin red/servidores     | Sin ejecutar                                          | SO, ejecutable usado, resultados antes/después  |
| GitHub Actions                  | Pendiente de publicación y ejecución remota           | Enlace al run y resultado de cada job           |
