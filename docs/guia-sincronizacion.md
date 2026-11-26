# Guía de respaldo y sincronización entre dispositivos

> **Estado:** guía del flujo previsto. En esta rama existen contratos de datos, pero el respaldo manual cifrado, los destinos Git/carpeta y sus pantallas aún no están integrados. Los nombres de controles son orientativos y deben ajustarse a la interfaz definitiva. No se ha ejecutado este procedimiento de extremo a extremo.

La sincronización del libro personal intercambia archivos entre dispositivos a través de un destino elegido por la persona. Es independiente de la sincronización de gastos grupales con el servidor. El destino compartido no sustituye una copia de seguridad conservada por separado.

## Antes de comenzar

1. Prepara los dos dispositivos con versiones compatibles de Cuentas Claras y acceso a sus libros locales. Conserva una copia adicional antes de una primera importación.
2. Elige una frase de paso de al menos 12 caracteres y guárdala en un lugar seguro. Debe ser exactamente la misma para abrir los archivos compartidos; si se pierde, no se pueden recuperar los datos cifrados con ella.
3. Elige un destino: una carpeta realmente compartida y accesible desde ambos dispositivos, o un repositorio Git privado al que ambos tengan acceso. No incluyas contraseñas ni tokens en la URL.
4. Usa datos de prueba para la primera verificación. Cuando esté disponible, la aplicación debe guardar los secretos de sincronización en el almacén seguro del sistema operativo, no en la carpeta ni en Git.

## Configurar una carpeta compartida

1. Crea o selecciona la carpeta compartida y espera a que esté disponible en ambos dispositivos. Comprueba permisos de lectura y escritura y que el servicio que la comparte haya terminado de transferir los archivos.
2. En cada dispositivo, abre la configuración de sincronización y selecciona **Carpeta compartida**.
3. Selecciona la ruta local que corresponde a esa misma carpeta en cada dispositivo. Las rutas pueden ser distintas aunque apunten al mismo destino.
4. Introduce la misma frase de paso y guarda la configuración. Sincroniza primero un dispositivo y luego el otro; espera a que la carpeta termine de actualizarse antes de cada operación.
5. Consulta el resultado y la fecha de la última sincronización exitosa. Si hay rechazos, revísalos antes de dar por concluido el proceso.

## Configurar Git privado

1. Crea un repositorio privado dedicado y concede acceso a ambos dispositivos. Ten disponibles la URL de clonación y las credenciales exigidas por el proveedor, sin incorporarlas a la URL ni a archivos del repositorio.
2. En cada dispositivo, abre la configuración de sincronización, selecciona **Git** y proporciona la misma URL del repositorio privado.
3. Introduce las credenciales y la misma frase de paso de sincronización; guarda la configuración. El repositorio debe permanecer privado aunque sus archivos de respaldo estén cifrados.
4. Sincroniza el primer dispositivo y después el segundo. Comprueba el reporte y la fecha de la última sincronización exitosa en cada uno. Ante cambios concurrentes, espera el resultado del reintento; si aparece un error, vuelve a intentar una vez recuperado el acceso.

## Exportar y restaurar en un dispositivo nuevo

1. En el dispositivo de origen, abre el flujo de respaldo manual, elige una ubicación y deja activado el cifrado. Introduce y confirma la frase de paso; guarda el archivo en un lugar accesible para el nuevo dispositivo.
2. Una exportación sin cifrar contiene información personal legible. Úsala solo si es necesaria, acepta la advertencia de la aplicación y protege y elimina las copias según corresponda.
3. En el dispositivo nuevo, abre el flujo de importación y selecciona el archivo. Si está cifrado, introduce exactamente la frase usada al exportarlo.
4. Revisa el resumen: eventos nuevos, eventos ya existentes y eventos rechazados con su motivo. Compara cuentas, saldos e historial con el origen; no des por completa la restauración si hay rechazos sin resolver.
5. Si también usarás sincronización continua, configura el mismo destino y frase en el dispositivo nuevo y sincroniza. Importar un respaldo debe fusionar los eventos, conservando los locales recientes.

## Problemas frecuentes

| Situación                                              | Qué hacer                                                                                                                                                          |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Frase incorrecta o archivo dañado                      | Comprueba que la frase coincide exactamente y vuelve a seleccionar una copia íntegra. Si persiste, usa otra copia conocida; no borres los datos locales.           |
| Versión de formato no soportada                        | Comprueba la versión de la aplicación y utiliza una versión compatible con el archivo. Conserva el archivo original.                                               |
| Destino inaccesible o sin conexión                     | Revisa la conexión, la URL o ruta y los permisos. En carpeta compartida, espera a que termine la transferencia y reintenta.                                        |
| Credenciales Git inválidas                             | Verifica el acceso al repositorio privado y actualiza las credenciales en la configuración. No las pegues en mensajes de soporte.                                  |
| Evento con identificador repetido y contenido distinto | Conserva el reporte y las copias; no fuerces la sustitución. Consulta al equipo con el identificador del evento, sin compartir información financiera ni secretos. |
| Falta un cambio en otro dispositivo                    | Revisa el resultado de ambas sincronizaciones, la conexión y que se use el mismo destino. Para carpeta, espera la transferencia y vuelve a sincronizar ambos.      |

Si una operación falla, verifica que los datos locales sigan presentes y conserva los archivos originales. La fecha de última sincronización exitosa no debe confundirse con un intento fallido.

## Referencias

- [ADR de cifrado propuesto](adr/019-cifrado-respaldo.md).
- [Matriz de validación y guion de demo](qa-sincronizacion-respaldo.md).
