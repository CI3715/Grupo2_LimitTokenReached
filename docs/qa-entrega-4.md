# QA de Entrega 4: monedas, tasas y patrimonio

Referencia: Especificación Técnica Entrega 4, secciones 2.1–2.5 y criterios de aceptación. La interfaz invoca los comandos Tauri y estos usan casos de uso y repositorios SQLite locales. El flujo completo de reinicio con red y servidores apagados todavía debe demostrarse manualmente.

## Estado de cobertura

La cobertura automatizada incluye validación de tasas, selección por fecha, inversión de tasas, falta de conversión y redondeo final en pruebas con repositorios fake en memoria. Las pruebas SQLite cubren persistencia y eliminación lógica, historial, selección de la última tasa del mismo día y creación de moneda propia.

| Área                   | Evidencia automatizada                                                                          | Pendiente                                                                                            |
| ---------------------- | ----------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| INV-02                 | Caso de uso impide total sin conversión; identifica moneda/par faltantes y no hace cruces       | Repetir y conservar evidencia visual durante la demostración                                        |
| INV-03                 | `Decimal` en dominio; pruebas de inversa y redondeo final                                       | Demostración visual del cálculo                                                                       |
| Validaciones           | Caso de uso valida fecha, fuente, monto, decimales, catálogo y monedas distintas               | Capturar errores de cada campo durante la demostración                                               |
| Fecha/tasa vigente     | Corte por fecha, preferencia directa, inversa, empate y exclusión de tasas eliminadas           | Verificar selección de tasas con datos del perfil de evaluación                                      |
| Persistencia/autonomía | Repositorios SQLite persisten tasas, moneda base e historial; pruebas locales                 | Cerrar y reabrir el ejecutable sin red ni servidores                                                 |
| Moneda propia          | Caso de uso valida catálogo y unicidad; SQLite permite usarla en una cuenta                    | Demostrar creación desde `/monedas` y selección al crear una cuenta                                  |

Los tests de selección y cálculo llaman los casos de uso productivos con repositorios fake; los tests SQLite comprueban los adaptadores de persistencia.

## Comandos automatizados

Desde la raíz del repositorio, con Rust/Cargo instalados:

```bash
cd app
cargo test -p contrato -p aplicacion -p dominio
cargo test -p aplicacion --test registrar_tasa_test --test consultar_patrimonio_test --test crear_moneda_test
cargo test -p infraestructura --test repositorio_finanzas_sqlite
cargo test -p infraestructura --test repositorio_preferencias_sqlite
```

El comando de casos de uso filtrados es una alternativa para revisar solo los grupos nuevos; esos tests también se ejecutan en el primero. Para regresión frontend, desde `app/`, con las dependencias instaladas:

```bash
pnpm run test:movements
pnpm run lint
pnpm run build
```

No ejecutar `cargo test --workspace` como prueba aislada sin servidor: incluye `conexion_servidor`, que hace una petición real a FastAPI. El CI selecciona explícitamente las suites locales y ejecuta las pruebas de movimientos frontend, lint y build. Los cambios de README y `docs/` también activan el workflow. Un CI verde con las suites actuales no certifica la Entrega 4 completa.

## Ejemplo 2.3 para la evaluación

Usar un perfil de evaluación sin otras cuentas ni tasas que alteren los totales. Conservar intacto el libro personal; no borrar su BD. El día de evaluación `D` debe ser igual o posterior al 21/10/2026. Consultar inicialmente `D`, de modo que las cuentas recién creadas existan al corte.

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

## Casos de dominio cubiertos por pruebas automatizadas

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

## Evidencia automatizada local

- Registrar la tasa valida los campos desde el caso de uso; el dominio contiene el tipo validado y sus errores.
- Patrimonio automatiza falta de tasa, selección por fecha, inversa y redondeo final con repositorios fake.
- Persistencia SQLite cubre tasas, moneda base, historial, eliminación lógica, moneda propia y cuentas en monedas propias.
- La interfaz dispone de `/tasas`, `/patrimonio` y `/monedas`; su verificación local requiere lint, tipos y build del frontend.
- El modo sin red tras cerrar y reabrir la aplicación no se ha ejecutado manualmente.

Estos resultados no incluyen la ejecución remota de GitHub Actions ni la evaluación manual de Entrega 4.

## Registro de evaluación

Completar tras integrar y ejecutar cada caso. No marcar como aprobado lo que esté bloqueado o sin ejecutar.

| Caso                            | Estado inicial                                        | Evidencia a completar                           |
| ------------------------------- | ----------------------------------------------------- | ----------------------------------------------- |
| Dominio E4 con fake             | Implementado; suites Rust locales                     | Guardar resultado de ejecución                  |
| Ejemplo 2.3                     | Implementado; falta demostración con perfil limpio    | Capturas 316.100 VES, aviso y 350 USD           |
| Historial, base y moneda propia | Implementado; falta demostración manual                | Capturas y comparación de saldos                |
| Reinicio sin red/servidores     | Sin ejecutar                                          | SO, ejecutable usado, resultados antes/después  |
| GitHub Actions                  | Pendiente de publicación y ejecución remota           | Enlace al run y resultado de cada job           |
