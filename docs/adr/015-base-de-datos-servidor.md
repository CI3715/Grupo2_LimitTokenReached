# ADR-015: Base de Datos del Servidor (SQLite)

**Fecha:** 2026-11-10
**Estado:** Aceptado  
**Decisores:** Grupo 2, Secc. 1 – LimitTokenReached  
**Etiquetas:** infraestructura, servidor, base-datos, SQLite, Entrega-5

## Contexto
La especificación técnica de la Entrega 5 establece explícitamente:
> *"Base de datos elegida por el equipo y justificada en una ADR (Guía Técnica, secciones 1.3 y 4.3)."*

Además, la sección 2.5 requiere:
> *"Por cada usuario se guarda: identificador, nombre de usuario, nombre visible, correo, hash de contraseñas y fecha de alta."*

Y la sección 2.5 también establece:
> *"El servidor no tiene tablas ni colecciones de cuentas, saldos, movimientos ni tasas (INV-16)."*

Esto busca garantizar que el servidor almacene únicamente los datos compartidos (usuarios, sesiones, y en entregas futuras: amistades, grupos, gastos compartidos), mientras que los datos financieros personales permanecen exclusivamente en el dispositivo del usuario.

## Decisión
Se adopta **SQLite** como base de datos del servidor FastAPI, gestionada a través del ORM **SQLAlchemy**.

**Justificación técnica:**
1. **Simplicidad y Cero Configuración:** No requiere instalar, configurar ni mantener un servidor de base de datos externo (como PostgreSQL o MySQL). Es un archivo local, lo que simplifica enormemente el entorno de desarrollo y el despliegue.
2. **Soporte Nativo en Python/SQLAlchemy:** SQLite está integrado en la biblioteca estándar de Python y es soportado de forma transparente por SQLAlchemy sin necesidad de instalar drivers adicionales (como `psycopg2` o `asyncpg`).
3. **Cumplimiento ACID:** SQLite garantiza transacciones ACID (especialmente habilitando el modo WAL - Write-Ahead Logging), lo cual es suficiente para asegurar la integridad de los registros de usuarios, hashes de contraseñas y sesiones.
4. **Escala Adecuada al Alcance:** Para un proyecto académico con un número limitado de usuarios concurrentes (el equipo de desarrollo y los evaluadores), el rendimiento de lectura/escritura de SQLite es más que suficiente.
5. **Alineación con la Arquitectura:** Dado que el servidor solo almacena metadatos de autenticación y relaciones sociales (no datos financieros pesados), la carga sobre la base de datos es mínima.

**Esquema inicial (Entrega 5, gestionado por SQLAlchemy):**
```sql
CREATE TABLE usuarios (
    id TEXT PRIMARY KEY, -- UUID generado por la aplicación o SQLAlchemy
    username VARCHAR(30) UNIQUE NOT NULL,
    nombre_visible VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    fecha_alta TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_usuarios_username ON usuarios(username);
CREATE INDEX idx_usuarios_email ON usuarios(email);
```

**Variables de entorno requeridas:**
- `DATABASE_URL`: URL de conexión a SQLite (ej. `sqlite:///./servidor.db` o `sqlite+aiosqlite:///./servidor.db` para asíncrono).
- `JWT_SECRET`: Secreto para firmar tokens JWT (nunca va en el código ni en el repositorio).

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|-------------|-------------------|
| **PostgreSQL** | Excelente para producción a gran escala, pero introduce una complejidad operativa innecesaria (instalación, configuración de usuarios, puertos) para el alcance actual de un proyecto académico que solo maneja metadatos de usuarios. |
| **MySQL** | Similar a PostgreSQL, requiere un servidor externo y no aporta beneficios sobre SQLite para este volumen de datos. |
| **MongoDB (NoSQL)** | Esquema flexible innecesario para datos altamente estructurados como usuarios. Menos garantías de integridad referencial nativa. |
| **Redis (solo en memoria)** | No persistente por defecto. Adecuado solo para caché o sesiones temporales, no para datos permanentes como el registro de usuarios. |

## Consecuencias

### Positivas
- **Despliegue inmediato:** Cualquier miembro del equipo puede clonar el repositorio y ejecutar el servidor sin instalar dependencias de base de datos externas.
- **Backups triviales:** Hacer una copia de seguridad es tan simple como copiar el archivo `.db`.
- **Ecosistema maduro:** SQLAlchemy proporciona todas las herramientas necesarias para migraciones (Alembic) y consultas seguras.
- **Cumplimiento de la rúbrica:** La especificación exige una ADR justificando la base de datos del servidor, y esta decisión está sólidamente fundamentada en el alcance del proyecto.

### Negativas / Restricciones
- **Concurrencia de escritura:** SQLite bloquea la base de datos durante escrituras pesadas. *Mitigación:* Habilitar el modo WAL (`PRAGMA journal_mode=WAL;`) y, dado el alcance del proyecto, la probabilidad de escrituras simultáneas conflictivas es extremadamente baja.
- **No es ideal para SaaS masivo:** Si el proyecto escalara a miles de usuarios reales, se debería migrar a PostgreSQL. Sin embargo, para la entrega académica, SQLite es la elección más pragmática y eficiente.

## Verificación (QA / CI)
- [ ] El servidor se conecta correctamente a la base de datos SQLite usando `DATABASE_URL`.
- [ ] Las pruebas automatizadas verifican que `username` y `email` son únicos (el servidor responde 409 en duplicados).
- [ ] Las pruebas automatizadas verifican que el hash de la contraseña no es recuperable.
- [ ] El README explica cómo configurar `DATABASE_URL` y `JWT_SECRET`.
- [ ] El archivo de base de datos `.db` y el archivo `.env` están en `.gitignore` y nunca se suben al repositorio.

## Referencias
- Especificación Técnica Entrega 5: Sección 2.5 (Persistencia y Seguridad en el Servidor).
- Guía Técnica: Secciones 1.3 y 4.3 (justificación de base de datos).
- SQLite Documentation: https://www.sqlite.org/index.html
- FastAPI + SQLAlchemy + SQLite: https://fastapi.tiangolo.com/tutorial/sql-databases/
- ADR-014: Autenticación local-servidor (JWT, argon2, keyring).
