# ADR-019: Cifrado de Respaldos y Sincronización entre Dispositivos

**Fecha:** 2026-11-26
**Estado:** Propuesto; pendiente de implementación y revisión por el equipo
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached
**Etiquetas:** respaldo, sincronizacion, criptografia, privacidad

## Contexto

El libro personal debe poder exportarse y viajar por una carpeta compartida o un repositorio Git privado sin revelar sus eventos, cuentas o saldos. La frase de paso puede tener menos entropía que una clave aleatoria; por ello se necesita una derivación costosa y un cifrado que detecte modificaciones. El formato plano del libro está descrito en [formato-exportacion.md](../formato-exportacion.md).

La sincronización del libro entre dispositivos es distinta de la sincronización de gastos grupales con el servidor descrita en [ADR-018](018-sincronizacion-idempotente-y-cursor.md). En la revisión de esta rama solo se encontraron contratos de respaldo, no el cifrado ni los transportes de este ADR. Los valores siguientes son una propuesta concreta para acordar con quienes implementan; no describen archivos ya producidos por la aplicación.

## Decisión propuesta

- Derivar una clave de 32 bytes a partir de la frase codificada en UTF-8, sin normalización ni modificación silenciosa, mediante **Argon2id versión 0x13**: memoria `m = 65536 KiB` (64 MiB), `t = 3` iteraciones y `p = 1` carril. Generar una sal aleatoria de 16 bytes por archivo mediante un generador criptográficamente seguro.
- Cifrar el contenido completo del formato plano con **XChaCha20-Poly1305**, clave de 32 bytes, nonce aleatorio de 24 bytes y etiqueta de autenticación de 16 bytes. Generar un nonce nuevo por archivo mediante un generador criptográficamente seguro; nunca reutilizar un par clave/nonce.
- La frase de paso no se guarda en el archivo ni en SQLite. Para sincronización, el almacenamiento persistente de secretos corresponde al almacén seguro del sistema operativo; credenciales Git y frase tampoco deben aparecer en logs ni en el repositorio de destino. La fuerza efectiva sigue dependiendo de la frase; el mínimo previsto en la interfaz es de 12 caracteres.
- Autenticar toda la cabecera exterior como datos asociados (AAD). Una versión desconocida se rechaza antes de intentar importar; una frase incorrecta, un archivo alterado o una etiqueta inválida producen el mismo error de autenticación. No aplicar eventos antes de autenticar y validar el archivo completo.

### Cabecera exterior propuesta, versión 1

Representación binaria en el orden indicado, sin relleno; enteros sin signo en orden de bytes de red (big endian). La cabecera completa de 54 bytes es el AAD exacto, seguido del texto cifrado y su etiqueta de 16 bytes al final. La fecha, la cantidad de eventos y los eventos mismos permanecen dentro del contenido cifrado, no en la cabecera exterior.

| Campo            | Bytes | Valor o significado      |
| ---------------- | ----: | ------------------------ |
| Identificador    |     4 | ASCII `CCBK`             |
| Versión exterior |     1 | `1`                      |
| KDF              |     1 | `1` = Argon2id v0x13     |
| Cifrado          |     1 | `1` = XChaCha20-Poly1305 |
| Memoria          |     4 | `65536`, en KiB          |
| Iteraciones      |     2 | `3`                      |
| Paralelismo      |     1 | `1`                      |
| Sal              |    16 | Aleatoria por archivo    |
| Nonce            |    24 | Aleatorio por archivo    |

La versión exterior identifica el sobre cifrado y es independiente de `VERSION=1.0` dentro del libro plano. Para esta versión se aceptan únicamente los identificadores y parámetros exactos anteriores: rechazar valores desconocidos antes de ejecutar el KDF evita aceptar combinaciones ambiguas o costes controlados por un archivo malicioso. Un cambio de parámetros o de representación requiere una versión exterior nueva y pruebas de compatibilidad. Rechazar cabeceras truncadas, texto cifrado sin etiqueta y datos de tamaño no permitido; definir límites de tamaño antes de publicar la implementación.

## Alternativas consideradas

| Alternativa                              | Motivo de la elección propuesta                                                                                         |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| AES-256-GCM                              | También es autenticado; XChaCha20-Poly1305 ofrece un nonce de 24 bytes apropiado para generación aleatoria por archivo. |
| Cifrado sin autenticación                | No detecta de forma fiable modificaciones del archivo.                                                                  |
| Hash rápido de la frase                  | Facilita ataques de diccionario; Argon2id incorpora coste de memoria y tiempo.                                          |
| Guardar secretos junto al libro o en Git | Expone los secretos a quien acceda al respaldo o destino.                                                               |

## Consecuencias y verificaciones pendientes

- Argon2id consume memoria y tiempo en cada exportación e importación; medirlo en los dispositivos soportados antes de aceptar la decisión. No reducir parámetros silenciosamente si falla la asignación.
- La pérdida de la frase hace irrecuperable el contenido cifrado; la aplicación debe advertirlo antes de exportar.
- El cifrado no oculta la existencia, tamaño aproximado ni fechas de los archivos en el sistema de archivos o Git.
- [ ] Revisar y acordar parámetros, cabecera, límites de tamaño y compatibilidad con el equipo de backend.
- [ ] Verificar ida y vuelta, salt y nonce distintos en exportaciones sucesivas, autenticación de cabecera y cuerpo, truncamiento y versión desconocida.
- [ ] Confirmar que errores y logs no revelan secretos y que ningún fallo altera la base de datos.

## Referencias

- [Formato de exportación](../formato-exportacion.md).
- [Guía de sincronización](../guia-sincronizacion.md).
- [Matriz de validación y demo](../qa-sincronizacion-respaldo.md).
