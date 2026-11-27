# Guía de respaldo y sincronización entre dispositivos

> **Estado de la funcionalidad:** el backend expone comandos Tauri para exportar e importar el libro, cifrado o plano, y sincronizarlo con una carpeta compartida. La interfaz de estos flujos, la configuración persistente, el transporte Git y el almacenamiento de sus secretos aún no están conectados en esta rama. Los pasos de pantalla siguientes son el flujo previsto y no una guía de controles ya disponibles. No se ha verificado manualmente de extremo a extremo.

La sincronización del libro personal entre dispositivos es independiente de la sincronización de gastos grupales con el servidor. Conserva una copia de seguridad separada del destino compartido.

## Antes de comenzar

1. Prepara dos instalaciones compatibles con libros y dispositivos independientes. Conserva una copia adicional antes de importar.
2. Elige una frase de paso de al menos 12 caracteres y consérvala en un lugar seguro. Debe coincidir exactamente al abrir el archivo; si se pierde, no se pueden recuperar los datos cifrados con ella.
3. Para carpeta compartida, elige una carpeta que ambos dispositivos puedan leer y escribir. Usa datos de prueba hasta completar la integración y validación.
4. No pongas contraseñas, tokens ni frases en archivos compartidos o mensajes. El almacenamiento seguro de secretos específico de esta función aún está pendiente.

## Respaldo y restauración: flujo previsto en la interfaz

1. En el dispositivo de origen, abre el futuro flujo de respaldo manual y selecciona la ubicación del archivo.
2. Elige cifrado e introduce y confirma la frase. El backend exige al menos 12 caracteres al exportar. Guarda el archivo en un lugar accesible para el dispositivo nuevo.
3. La exportación sin cifrar contiene información personal legible. Úsala solo cuando sea necesaria y protege la copia.
4. En el dispositivo nuevo, abre el futuro flujo de importación, selecciona el archivo y, si está cifrado, proporciona la misma frase.
5. Revisa los eventos nuevos, existentes y rechazados; compara cuentas, saldos e historial con el origen. Un rechazo requiere revisión antes de dar por completada la restauración.

Los comandos disponibles son `exportar_libro` e `importar_libro`; aún no se encontraron pantallas que los invoquen. La importación fusiona eventos y no está diseñada para borrar eventos locales ausentes en el archivo.

## Carpeta compartida: flujo previsto en la interfaz

1. Prepara una carpeta compartida accesible desde ambas instalaciones y verifica permisos de lectura y escritura. Las rutas locales pueden ser distintas si apuntan al mismo destino.
2. Cuando exista la configuración en pantalla, selecciona la carpeta y usa la misma frase en ambos dispositivos. Espera a que el servicio de carpeta termine de transferir los archivos antes de cada operación.
3. Sincroniza el primer dispositivo, luego el segundo y nuevamente el primero para completar el intercambio. Consulta los eventos enviados, recibidos y rechazados.
4. Si hay rechazos, conserva los archivos y revisa el reporte. No elimines ni edites manualmente los lotes para resolver un conflicto.

El comando existente `sincronizar_carpeta_libro` recibe la ruta y frase por llamada; lee lotes `.ccbak` recursivamente y publica lotes cifrados bajo el identificador del dispositivo. Todavía no existe en esta rama una pantalla de configuración o sincronización automática del libro.

## Git privado: pendiente

La configuración de un repositorio Git privado, sus credenciales, clonación y sincronización aún no están implementadas en el backend revisado. Cuando se incorporen se deberá completar aquí el procedimiento real, sin incluir credenciales en la URL ni en el repositorio. No se puede utilizar este método desde la aplicación actual.

## Problemas frecuentes

| Situación                               | Qué hacer                                                                                                                                |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Frase incorrecta o archivo dañado       | Comprueba la frase y usa una copia íntegra. Conserva el libro local y el archivo original.                                               |
| Versión no soportada                    | Verifica la versión de la aplicación y conserva el archivo hasta disponer de una versión compatible.                                     |
| Carpeta inaccesible                     | Revisa ruta, permisos, conexión y transferencia del servicio de carpeta; vuelve a intentar.                                              |
| Evento rechazado por contenido distinto | Conserva el reporte y las copias; consulta al equipo con el identificador, sin compartir secretos ni datos financieros.                  |
| Falta un cambio en otro dispositivo     | Comprueba que ambas instalaciones utilicen la misma carpeta y frase, que la transferencia haya terminado y que ambas hayan sincronizado. |

El catálogo de errores contempla además credenciales Git inválidas, pero el transporte Git y su conexión a la interfaz aún están pendientes. Los mensajes y nombres definitivos de los controles deberán verificarse al integrar el frontend.

## Referencias

- [ADR de cifrado](adr/019-cifrado-respaldo.md).
- [Matriz de validación y guion de demo](qa-sincronizacion-respaldo.md).
