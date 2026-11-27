# Guía de respaldo y sincronización entre dispositivos

La pantalla **Respaldo** permite exportar e importar el libro y sincronizarlo por carpeta compartida o por un repositorio Git. Estos flujos están integrados en el código; la validación manual en dispositivos fue reportada como exitosa. La sincronización del libro es independiente de la sincronización de gastos grupales con el servidor.

## Antes de comenzar

- Conserva una copia adicional de tus datos antes de restaurar o sincronizar por primera vez.
- Elige una frase de paso de al menos 12 caracteres y guárdala en un lugar seguro. Si la pierdes, no podrás recuperar el respaldo cifrado. Utiliza exactamente la misma frase en los dispositivos que comparten un destino.
- No escribas credenciales en la URL del repositorio. La aplicación utiliza el gestor de credenciales configurado para Git en cada dispositivo.
- La aplicación recuerda el tipo y la ruta o URL del destino en el almacenamiento local de la interfaz. La frase de sincronización se mantiene en la página, no se guarda para futuras sesiones: deberás introducirla de nuevo al volver.

## Exportar y restaurar un respaldo

1. Abre **Respaldo** y busca **Exportar copia de seguridad**. Escribe la carpeta de destino o la ruta completa de un archivo `.ccbak`; si indicas una carpeta, se usa el nombre `respaldo.ccbak`.
2. Selecciona **Cifrar**, introduce la frase y repítela. Si eliges **No cifrar**, la copia contiene información financiera legible y debes protegerla.
3. Pulsa **Exportar copia** y lleva el archivo al dispositivo nuevo.
4. En **Importar copia de seguridad**, selecciona el archivo o escribe su ruta. Introduce la frase si la copia está cifrada y pulsa **Importar copia**.
5. Revisa el resumen de eventos nuevos, existentes y rechazados. Compara cuentas, saldos e historial con el origen y revisa cualquier rechazo antes de dar por terminada la restauración.

Si el selector no entrega una ruta completa, la pantalla solicita escribirla manualmente. Importar fusiona los eventos con los locales.

## Sincronizar con carpeta compartida

1. Prepara una carpeta accesible con permisos de lectura y escritura desde ambos dispositivos. Las rutas locales pueden ser diferentes si apuntan a la misma carpeta.
2. En **Respaldo**, selecciona **Carpeta compartida** y escribe la ruta. Si el selector no proporciona la ruta correcta, escríbela manualmente.
3. Introduce la frase común y pulsa **Guardar destino y sincronizar**. Espera a que el servicio de carpeta termine de transferir los archivos.
4. Repite en el segundo dispositivo y vuelve a sincronizar el primero. Consulta el reporte y los rechazos.

## Sincronizar con Git privado

1. Prepara un repositorio privado dedicado y instala y configura Git en cada dispositivo, incluyendo nombre y correo para sus commits, y habilita el acceso al repositorio mediante el gestor de credenciales o el método de autenticación configurado. Git se ejecuta sin solicitud interactiva en terminal, por lo que el acceso debe estar disponible previamente.
2. En **Respaldo**, selecciona **Repositorio Git** e introduce la URL sin credenciales y la frase común.
3. Pulsa **Guardar destino y sincronizar** en el primer dispositivo, después en el segundo y finalmente vuelve a sincronizar el primero. Revisa el reporte en cada uno.
4. Si Git rechaza las credenciales, actualízalas en su gestor en ese dispositivo y reintenta. La aplicación no recibe credenciales en el formulario.

La sincronización se inicia con **Guardar destino y sincronizar** o **Sincronizar ahora**; actualmente no se ejecuta automáticamente al abrir la aplicación. La fecha y el reporte de la última sincronización exitosa se muestran durante la sesión de la pantalla y no se conservan al volver a abrirla.

## Problemas frecuentes

| Situación                               | Qué hacer                                                                                                               |
| --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Frase incorrecta o archivo dañado       | Comprueba la frase y utiliza una copia íntegra. Conserva el archivo original y los datos locales.                       |
| Versión no soportada                    | Verifica la versión de la aplicación y conserva el archivo hasta disponer de una versión compatible.                    |
| Destino inaccesible                     | Comprueba la ruta o URL, conexión y permisos. En carpeta, espera a que termine la transferencia.                        |
| Credenciales Git rechazadas             | Actualízalas en el gestor de credenciales de Git y vuelve a sincronizar.                                                |
| Evento rechazado por contenido distinto | Conserva el reporte y las copias; consulta al equipo con el identificador, sin compartir secretos ni datos financieros. |
| Falta un cambio                         | Comprueba destino y frase, espera la transferencia y sincroniza ambos dispositivos de nuevo.                            |

## Referencias

- [ADR de cifrado](adr/019-cifrado-respaldo.md).
- [Matriz de validación y demo](qa-sincronizacion-respaldo.md).
