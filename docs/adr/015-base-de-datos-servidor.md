# ADR-015: Base de Datos del Servidor (PostgreSQL)

**Fecha:** 2026-11-10  
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** infraestructura, servidor, base-datos, PostgreSQL, Entrega-5

## Contexto
La especificación técnica de la Entrega 5 establece explícitamente que:
> *"Base de datos elegida por el equipo y justificada en una ADR (Guía Técnica, secciones 1.3 y 4.3)."*

Además, la sección 2.5 requiere:
> *"Por cada usuario se guarda: identificador, nombre de usuario, nombre visible, correo, hash de la contraseña y fecha de alta."*

Y la sección 2.5 también establece:
> *"El servidor no tiene tablas ni colecciones de cuentas, saldos, movimientos ni tasas (INV-16)."*

Esto busca garantizar que el servidor almacene únicamente los datos compartidos (usuarios, sesiones, y en entregas futuras: amistades, grupos, gastos compartidos), mientras que los datos financieros personales permanecen exclusivamente en el dispositivo del usuario.

## Decisión
Se adopta **PostgreSQL** como base de datos del servidor FastAPI.

**Justificación técnica:**
- **Confiabilidad y madurez:** PostgreSQL es un sistema de gestión de bases de datos relacional de código abierto con más de 30 años de desarrollo, ampliamente utilizado en producción.
- **Soporte para JSONB:** permite almacenar datos semiestructurados si es necesario en el futuro (ej. metadatos de grupos).
- **Transacciones ACID:** garantiza la integridad de los datos en operaciones críticas como el registro de usuarios y la gestión de sesiones.
- **Índices avanzados:** soporta índices únicos para garantizar la unicidad de `username` y `email` (RF-SOC-01).
- **Escalabilidad:** puede manejar miles de usuarios concurrentes, necesario para las funciones sociales de las entregas futuras.
- **Compatibilidad con FastAPI:** existen drivers maduros como `asyncpg` y `SQLAlchemy` que se integran perfectamente con FastAPI.

**Esquema inicial (Entrega 5):**
```sql
CREATE TABLE usuarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(30) UNIQUE NOT NULL,
    nombre_visible VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    fecha_alta TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_usuarios_username ON usuarios(username);
CREATE INDEX idx_usuarios_email ON usuarios(email);
```

### Variables de entorno requeridas

- `DATABASE_URL`: URL de conexión a PostgreSQL (ej. `postgresql://user:password@localhost:5432/cuentas_claras`)
- `JWT_SECRET`: Secreto para firmar tokens JWT (nunca va en el código ni en el repositorio)

### Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| **SQLite en el servidor** | No soporta concurrencia de escritura multi-usuario. Adecuado solo para aplicaciones de un solo usuario o lectura intensiva. |
| **MySQL** | Menos características avanzadas que PostgreSQL (ej. JSONB, índices parciales). Comunidad más pequeña en el ecosistema Python. |
| **MongoDB (NoSQL)** | Esquema flexible innecesario para datos estructurados como usuarios. Menos garantías de integridad referencial. |
| **Redis (solo en memoria)** | No persistente. Adecuado solo para caché o sesiones temporales, no para datos permanentes como usuarios. |
| **Supabase (PostgreSQL gestionado)** | Dependencia de un servicio externo. Para el entorno local de desarrollo, PostgreSQL local es más simple y no requiere cuenta externa. |

### Consecuencias

#### Positivas
- **Integridad de datos:** las restricciones `UNIQUE` en `username` y `email` garantizan que no haya duplicados a nivel de base de datos.
- **Seguridad:** el hash de contraseñas con argon2 (ver ADR-014) se almacena de forma segura.
- **Escalabilidad:** puede crecer para soportar las funciones sociales de las entregas 6 y posteriores.
- **Ecosistema maduro:** herramientas como pgAdmin, DBeaver y migraciones con Alembic facilitan el desarrollo.
- **Cumplimiento de la rúbrica:** la especificación exige una ADR justificando la base de datos del servidor.

#### Negativas / Restricciones
- Requiere instalar y configurar PostgreSQL localmente para desarrollo.
- Mayor complejidad operativa que SQLite (backups, mantenimiento).
- Necesita variables de entorno para la conexión (mitigado con un archivo `.env` local, nunca subido al repositorio).

### Verificación (QA / CI)

- [ ] El servidor se conecta correctamente a PostgreSQL usando `DATABASE_URL`.
- [ ] Las pruebas automatizadas verifican que `username` y `email` son únicos (409 en duplicados).
- [ ] Las pruebas automatizadas verifican que el hash de la contraseña no es recuperable.
- [ ] El README explica cómo configurar `DATABASE_URL` y `JWT_SECRET`.
- [ ] El archivo `.env` está en `.gitignore` y nunca se sube al repositorio.

### Referencias

- Especificación Técnica Entrega 5: Sección 2.5 (Persistencia y Seguridad en el Servidor).
- Guía Técnica: Secciones 1.3 y 4.3 (justificación de base de datos).
- PostgreSQL Documentation: https://www.postgresql.org/docs/
- FastAPI + SQLAlchemy + PostgreSQL: https://fastapi.tiangolo.com/tutorial/sql-databases/
- ADR-014: Autenticación local-servidor (JWT, argon2, keyring).
