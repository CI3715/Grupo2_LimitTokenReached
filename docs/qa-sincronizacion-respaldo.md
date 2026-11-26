# Matriz de validación y demo: respaldo y sincronización del libro

**Estado:** guion preparado; casos pendientes de ejecución. Este documento cubre la verificación solicitada para las secciones 2.8 y 3 del reparto. No se dispone aquí del texto completo de esas secciones ni de los datos del escenario 2.6 de esta funcionalidad. Los 44 eventos son el resultado esperado facilitado en el reparto; el saldo exacto y la composición del escenario deben obtenerse de la especificación o fixture oficial antes de ejecutar la demo. No inferirlos ni confundirlos con otro escenario 2.6 del repositorio.

La sincronización de este documento es la del libro personal entre dispositivos, no la de gastos grupales con el servidor. Las pruebas automatizadas de motor, criptografía y transporte pertenecen a sus responsables; esta matriz organiza la validación y no sustituye sus suites.

## Preparación y evidencia

- [ ] Registrar fecha, versión/commit probado, sistema operativo y resultado de cada caso: **pendiente**, **aprobado**, **fallido** o **bloqueado**, con incidencia cuando aplique. No marcar aprobado sin ejecutarlo.
- [ ] Preparar dos instalaciones aisladas, Laptop (L) y Teléfono (T), con bases de datos y `device_id` diferentes. El nombre Teléfono identifica la segunda instancia: si esa plataforma aún no está soportada, usar una segunda instalación compatible y dejar constancia.
- [ ] Preparar una frase de prueba, un destino compartido aislado y un repositorio Git privado de prueba. Nunca adjuntar frases, tokens, contenido financiero real ni capturas de secretos a la evidencia.
- [ ] Obtener la fixture oficial del escenario 2.6, su saldo exacto por cuenta y moneda y los conjuntos iniciales de eventos. Registrar también los identificadores y contenido esperados para comparar, no solo los totales. El escenario debe permitir comprobar una unión de 44 eventos únicos.
- [ ] Registrar antes y después el conjunto de eventos (ID y contenido), los saldos exactos por cuenta/moneda, el historial y los reportes. Comparar sin coma flotante; usar datos sintéticos.
- [ ] Configurar una observación de red que cubra las solicitudes de la aplicación al servidor de Cuentas Claras durante la ventana de prueba y registrar método, destino, momento y número de solicitudes, sin guardar secretos ni datos sensibles. Comprobar que la instrumentación detecta una solicitud de control antes de medir. No confundir el tráfico necesario hacia Git o el servicio de carpeta con tráfico al servidor de Cuentas Claras.

## Matriz de validación

| Caso                        | Preparación y acción                                                                        | Resultado esperado                                                                                                                                      | Estado    |
| --------------------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| Exportación cifrada         | Exportar el libro de L con una frase de prueba.                                             | Se genera un archivo cifrado; no aparecen frase, credenciales ni datos legibles del libro. Registrar ruta de evidencia sin incluir el archivo sensible. | Pendiente |
| Restauración                | Importar el archivo en una base limpia y comparar con L.                                    | Mismo conjunto de eventos, historial y saldos exactos por cuenta y moneda.                                                                              | Pendiente |
| Frase incorrecta            | Registrar estado de una base con datos locales; importar con otra frase.                    | Error amigable; eventos, historial y saldos no cambian.                                                                                                 | Pendiente |
| Archivo alterado o truncado | Modificar una copia del archivo cifrado o truncarla e intentar importar.                    | Error de autenticación o archivo inválido; base sin cambios.                                                                                            | Pendiente |
| Versión no soportada        | Usar un archivo de prueba con versión exterior no soportada.                                | Error de versión; base sin cambios.                                                                                                                     | Pendiente |
| Importación doble           | Importar dos veces el mismo archivo en una base conocida.                                   | La segunda importación no agrega eventos ni altera saldos; reporte coherente de existentes.                                                             | Pendiente |
| Conflicto aislado           | Importar un lote con un ID existente y contenido distinto junto con un evento nuevo válido. | Se rechaza y reporta el conflicto; el nuevo válido se incorpora una sola vez; el evento local original se conserva.                                     | Pendiente |
| Respaldo antiguo o parcial  | Crear un evento local reciente y después importar un archivo que no lo contiene.            | El evento reciente permanece y el historial se fusiona sin eliminación.                                                                                 | Pendiente |
| Convergencia 2.6            | Ejecutar ambos órdenes desde copias idénticas de la fixture, como se indica en el guion.    | Ambos resultados contienen los mismos 44 eventos únicos con igual contenido y el saldo exacto oficial por cuenta/moneda.                                | Pendiente |
| Gasto grupal                | Aplicar en dos dispositivos el mismo gasto grupal y sincronizar el libro.                   | Una sola aplicación por identidad del evento; sin duplicar movimiento ni saldo.                                                                         | Pendiente |
| Transporte                  | Repetir la sincronización usando carpeta compartida y Git privado.                          | Ambos dispositivos convergen y el reporte refleja envíos, recepciones y rechazos.                                                                       | Pendiente |
| Privacidad de red           | Medir únicamente durante exportación, importación y sincronización del libro.               | Cero solicitudes al servidor de Cuentas Claras durante esas operaciones. Registrar por separado el tráfico hacia el destino elegido.                    | Pendiente |

## Guion de demostración en dos instancias

1. **Estado inicial:** cargar en L y T los datos oficiales del escenario 2.6; registrar los eventos y saldos iniciales. Preparar una copia de esas bases para reiniciar el ensayo de cada orden. Desactivar cualquier sincronización grupal automática durante la medición y anotar cualquier otra actividad de la app que pueda generar solicitudes al servidor.
2. **Exportar y restaurar:** iniciar la captura de red, exportar L cifrado e importar en una base limpia. Comparar eventos, historial y saldos exactos con L. Detener y guardar el resumen de la captura.
3. **Frase errónea:** conservar el estado de la base receptora, intentar importar con una frase incorrecta y verificar el error y la ausencia de cambios. Repetir con una copia alterada y con versión no soportada.
4. **Importar doble:** importar el archivo correcto dos veces; mostrar ambos reportes y demostrar que el segundo intento no agrega eventos ni cambia saldos.
5. **Orden L → T:** restaurar las bases iniciales del escenario y usar un destino de prueba vacío. Sincronizar L, esperar la disponibilidad del lote, sincronizar T y volver a sincronizar L hasta completar el intercambio. Guardar el conjunto de eventos y saldos de ambas instancias.
6. **Orden T → L:** restaurar las mismas bases iniciales y usar otro destino vacío. Sincronizar T, esperar el lote, sincronizar L y volver a sincronizar T hasta completar el intercambio. Comparar ambas instancias con el resultado del paso anterior y con la fixture oficial: exactamente 44 eventos únicos, contenido idéntico y saldos exactos esperados.
7. **Privacidad:** para cada operación medida mostrar la ventana y el contador de solicitudes hacia el servidor de Cuentas Claras. El resultado exigido es cero; cualquier solicitud durante la ventana debe investigarse, no descartarse sin evidencia. El acceso al destino Git o carpeta se registra por separado.
8. **Cierre:** registrar resultados, bloqueos e incidencias. No presentar un paso bloqueado o una prueba automatizada aún inexistente como aprobada.

## Dependencias abiertas

- Implementación y conexión de cifrado, importación/exportación, transportes, almacenamiento de secretos y pantallas.
- Fixture y saldo oficial exacto del escenario 2.6 de sincronización entre dispositivos; texto completo de las secciones 2.8 y 3 para comprobar cobertura literal.
- Acuerdo del equipo sobre el [ADR de cifrado propuesto](adr/019-cifrado-respaldo.md) y pruebas en las plataformas soportadas.

## Referencias

- [Guía de usuario](guia-sincronizacion.md).
- [Formato de exportación](formato-exportacion.md).
- [ADR-018: sincronización de gastos de grupo](adr/018-sincronizacion-idempotente-y-cursor.md).
