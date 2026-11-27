# ADR-019: Cifrado de Respaldos y Sincronización entre Dispositivos

**Fecha:** 2026-11-26

**Estado:** Implementado en backend e integrado con la interfaz; validado de extremo a extremo

**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached

**Etiquetas:** respaldo, sincronizacion, criptografia, privacidad, Entrega-8

## Contexto

El libro personal necesita poder exportarse y transferirse sin exponer sus eventos, cuentas o saldos. La frase de paso no es una clave aleatoria; se deriva una clave con una función costosa y se utiliza cifrado autenticado para detectar modificaciones. El formato del libro antes del cifrado se describe en [formato-exportacion.md](../formato-exportacion.md).

La sincronización del libro entre dispositivos es distinta de la sincronización de gastos grupales con el servidor descrita en [ADR-018](018-sincronizacion-idempotente-y-cursor.md). Este ADR documenta el cifrado implementado en `app/crates/infraestructura/src/criptografia_respaldo.rs`. La interfaz y los transportes de carpeta y Git están conectados; la validación manual fue reportada como exitosa.

## Decisión e implementación

- Se deriva una clave de **32 bytes** con **Argon2id v0x13**, memoria `m = 65536 KiB` (64 MiB), `t = 3` iteraciones y `p = 4` carriles. La frase se pasa como bytes UTF-8, sin normalización en el cifrador. La sal es de **32 bytes**, generada de forma aleatoria para cada exportación mediante `OsRng`.
- Se utiliza **AES-256-GCM**, con nonce aleatorio de **12 bytes** por exportación y etiqueta de autenticación de **16 bytes**. El cifrador requiere al menos 12 caracteres al exportar y libera la clave derivada mediante `Zeroizing`.
- Se cifra el libro plano completo. La cabecera exterior se autentica como datos asociados (AAD), incluyendo el salto de línea después de `NONCE`. Una frase incorrecta o un fallo de autenticación devuelve el mensaje «Frase de paso incorrecta o archivo dañado.».
- El serializador filtra campos de credenciales y tokens y enmascara cadenas con forma de JWT. Esto no garantiza detectar cualquier secreto arbitrario escrito en texto libre; no se deben incluir secretos en los datos del libro.
- El caso de uso de importación descifra y deserializa antes de fusionar eventos. El rechazo de una versión no soportada ocurre antes del descifrado; los comandos actualmente devuelven errores como `String`, aunque existe un enum de errores tipados en el crate de contratos.

### Cabecera exterior existente

El archivo es texto UTF-8; cada línea termina en `\n`. `SALT`, `NONCE` y `DATOS` usan Base64 estándar. `DATOS` contiene el texto cifrado seguido de la etiqueta GCM. Se muestra la estructura, no valores reales:

```text
CUENTASCLARAS-BACKUP
FORMATO=1
VERSION=1.0
CIFRADO=AES-256-GCM
KDF=ARGON2ID
PARAMETROS=M=65536,T=3,P=4
SALT=<base64 de 32 bytes>
NONCE=<base64 de 12 bytes>
DATOS=<base64 del texto cifrado y etiqueta>
```

La cabecera autenticada comprende desde `CUENTASCLARAS-BACKUP\n` hasta `NONCE=...\n`, sin incluir `DATOS=`. La fecha, el total y los eventos del formato plano quedan dentro del contenido cifrado. El lector comprueba `FORMATO`, `VERSION`, algoritmo, KDF, parámetros y longitudes de sal y nonce. `FORMATO` y `VERSION` son campos distintos; el serializador plano también utiliza `VERSION=1.0`.

## Alternativas consideradas

| Alternativa                | Consideración                                                                          |
| -------------------------- | -------------------------------------------------------------------------------------- |
| XChaCha20-Poly1305         | También ofrece cifrado autenticado; la implementación actual utiliza AES-256-GCM.      |
| Cifrado sin autenticación  | No ofrece la misma protección frente a modificaciones del respaldo.                    |
| Derivación con hash rápido | Facilita ataques de diccionario frente a una función costosa en memoria como Argon2id. |

## Consecuencias y pendientes

- La pérdida de la frase impide recuperar el contenido cifrado. El cifrado no oculta la existencia ni el tamaño aproximado del archivo.
- Existen comandos Tauri para exportar, importar y sincronizar mediante carpeta. También existen transporte Git y pantalla de respaldo: la URL o ruta del destino se guarda en localStorage y las credenciales Git se resuelven mediante el helper de Git. La frase de sincronización permanece en la página y no se guarda en el keyring; el keyring existente corresponde a la sesión.
- Las pruebas automatizadas cubren, entre otros casos, ida y vuelta, frase incorrecta, alteración, versión no soportada y ausencia de cambios en la base. La matriz recoge la validación manual reportada como exitosa.
- La política de límites de tamaño de archivo y la medición del coste de Argon2 en dispositivos soportados quedan pendientes de evaluación.

## Referencias

- [Implementación del cifrador](../../app/crates/infraestructura/src/criptografia_respaldo.rs).
- [Formato de exportación](../formato-exportacion.md).
- [Guía de sincronización](../guia-sincronizacion.md) y [matriz de validación](../qa-sincronizacion-respaldo.md).
