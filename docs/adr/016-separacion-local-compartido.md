### ADR-016: Separación entre Datos Locales y Compartidos

**Fecha:** 2026-11-10  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** arquitectura, local-first, servidor, INV-16, RES-02, RF-SOC-04, Entrega-6

## Contexto
La especificación técnica de la Entrega 6 señala lo siguiente:
> *"Lo compartido vive en el servidor; lo personal, no. Amistades, grupos, miembros y gastos de grupo se guardan en el servidor porque varias personas los ven. Las cuentas, movimientos, saldos y tasas del libro personal siguen siendo solo locales y no viajan al servidor (INV-16, RES-02)."*

Y la sección 2.4 de la Entrega 4 ya había establecido:
> *"Las tasas son personales: no se envían al servidor (INV-16, RNF-01)."*

También la sección 2.5 de la Entrega 5 reitera:
> *"El servidor no tiene tablas ni colecciones de cuentas, saldos, movimientos ni tasas (INV-16)."*

Esto busca garantizar que los datos financieros personales del usuario nunca salgan de su dispositivo sin su autorización explícita, mientras que los datos sociales (amistades, grupos, gastos compartidos) requieren un servidor para la coordinación entre múltiples usuarios.

## Decisión
Se adopta una **arquitectura híbrida** donde el sistema se divide en dos dominios claramente separados:

**Dominio Local (Rust + SQLite embebido):**
- Cuentas del libro personal
- Movimientos (ingresos, gastos, ajustes)
- Saldos calculados
- Tasas de cambio
- Monedas (catálogo y propias)
- Categoría de movimientos

**Dominio Compartido (Servidor FastAPI + PostgreSQL):**
- Usuarios registrados
- Amistades e invitaciones
- Grupos y miembros
- Gastos compartidos
- Balances de grupo

La aplicación cliente (Tauri + Next.js) se comunica con el servidor mediante HTTP/REST usando tokens de sesión (ADR-014). Las peticiones al servidor **nunca** contienen datos del libro personal (cuentas, movimientos, saldos, tasas).

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| Sincronizar todo al servidor (incluyendo datos personales) | Viola INV-16 y RF-SOC-04. Los datos financieros personales nunca deben salir del dispositivo sin autorización explícita. |
| Mantener todo local (sin servidor) | Imposible: las amistades, grupos y gastos compartidos requieren coordinación entre múltiples usuarios en tiempo real. |
| Usar peer-to-peer sin servidor | Mayor complejidad, problemas de disponibilidad y sincronización. El servidor centralizado es más simple y cumple los requisitos. |
| Usar el mismo SQLite local para datos compartidos | No permite la coordinación entre dispositivos de diferentes usuarios. |

## Consecuencias

### Positivas
- **Privacidad garantizada (INV-16):** los datos financieros personales nunca salen del dispositivo.
- **Funcionamiento offline:** todo lo personal sigue funcionando sin conexión ni servidor (RNF-01, RNF-17).
- **Coordinación multi-usuario:** las funciones sociales requieren servidor, pero solo esos datos viajan por la red.
- **Arquitectura clara:** cada tipo de dato tiene un lugar definido, evitando ambigüedades en el desarrollo.
- **Cumplimiento de la rúbrica:** separa explícitamente lo local de lo compartido, como exige la especificación.

### Negativas / Restricciones
- Requiere mantener dos sistemas de persistencia (SQLite local + PostgreSQL del servidor).
- La aplicación debe manejar dos modos de operación: con y sin conexión.
- Las funciones sociales requieren sesión y conexión; sin ellas, solo funciona lo personal.

## Verificación (QA / CI)
- [ ] Ninguna petición HTTP al servidor contiene datos del libro personal (cuentas, movimientos, saldos, tasas).
- [ ] Las pruebas automatizadas verifican que las funciones personales funcionan con la red desactivada y el servidor apagado.
- [ ] Las pruebas automatizadas verifican que las funciones sociales requieren sesión y conexión.
- [ ] El README explica claramente qué datos son locales y cuáles son compartidos.

## Referencias
- Especificación Técnica Entrega 6: Sección 1 (Objetivo General) y Sección 2.4 (Almacenamiento Local).
- Especificación Técnica Entrega 5: Sección 2.5 (Persistencia y Seguridad en el Servidor).
- Especificación Técnica Entrega 4: Sección 2.4 (Almacenamiento Local).
- INV-16: Los datos personales no viajan al servidor.
- RES-02: Lo compartido vive en el servidor; lo personal, no.
- ADR-005: Local-First y Event Sourcing.
- ADR-014: Autenticación local-servidor (JWT, argon2, keyring).
- ADR-015: Base de Datos del Servidor (PostgreSQL).
