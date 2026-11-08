# ADR-014: Persistencia de cuentas de autenticación en el servidor

- **Estado:** Aceptado
- **Fecha:** 2026-11-08
- **Contexto:** Entrega 5 requiere registrar cuentas y emitir tokens, manteniendo
  la aplicación local-first.

## Decisión

El servidor usa SQLite mediante SQLAlchemy para almacenar únicamente usuarios
de autenticación. La ruta del archivo se configura con `DATABASE_PATH` y, si es
relativa, se resuelve desde la raíz del repositorio. La tabla almacena
identificador, nombre de usuario, nombre visible, correo normalizado, hash
Argon2id y fecha de alta. Las restricciones únicas de la base de datos refuerzan
la validación previa de correo y nombre de usuario.

El secreto JWT se lee desde `JWT_SECRET`; nunca se almacena en la base de datos
ni en el cliente. Los tokens vencen a los 60 minutos.

## Límites de privacidad

La base SQLite del servidor es distinta del libro SQLite de Tauri. El servidor
no recibe ni persiste cuentas financieras, movimientos, saldos o tasas. La
selección de SQLite corresponde al entorno local de esta entrega; un despliegue
futuro requeriría almacenamiento persistente administrado o migrar a otro motor.
