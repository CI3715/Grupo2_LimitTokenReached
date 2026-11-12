PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

CREATE TABLE IF NOT EXISTS amistades (
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    emisor_id VARCHAR(36) NOT NULL,
    destinatario_id VARCHAR(36) NOT NULL,
    estado VARCHAR(16) NOT NULL,
    creada_en DATETIME NOT NULL,
    FOREIGN KEY (emisor_id) REFERENCES usuarios(id),
    FOREIGN KEY (destinatario_id) REFERENCES usuarios(id)
);

CREATE INDEX IF NOT EXISTS ix_amistades_emisor_id
    ON amistades (emisor_id);
CREATE INDEX IF NOT EXISTS ix_amistades_destinatario_id
    ON amistades (destinatario_id);
CREATE INDEX IF NOT EXISTS ix_amistades_estado
    ON amistades (estado);

CREATE TABLE IF NOT EXISTS grupos (
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    nombre VARCHAR(120) NOT NULL,
    descripcion VARCHAR(1000),
    moneda VARCHAR(3) NOT NULL,
    actividad VARCHAR(200),
    creador_id VARCHAR(36) NOT NULL,
    creado_en DATETIME NOT NULL,
    FOREIGN KEY (creador_id) REFERENCES usuarios(id)
);

CREATE TABLE IF NOT EXISTS miembros_grupo (
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    grupo_id VARCHAR(36) NOT NULL,
    usuario_id VARCHAR(36) NOT NULL,
    rol VARCHAR(16) NOT NULL,
    estado VARCHAR(16) NOT NULL,
    invitado_por_id VARCHAR(36) NOT NULL,
    creada_en DATETIME NOT NULL,
    CONSTRAINT uq_miembro_grupo_usuario UNIQUE (grupo_id, usuario_id),
    FOREIGN KEY (grupo_id) REFERENCES grupos(id),
    FOREIGN KEY (usuario_id) REFERENCES usuarios(id),
    FOREIGN KEY (invitado_por_id) REFERENCES usuarios(id)
);

CREATE INDEX IF NOT EXISTS ix_miembros_grupo_grupo_id
    ON miembros_grupo (grupo_id);
CREATE INDEX IF NOT EXISTS ix_miembros_grupo_usuario_id
    ON miembros_grupo (usuario_id);
CREATE INDEX IF NOT EXISTS ix_miembros_grupo_estado
    ON miembros_grupo (estado);

CREATE TABLE IF NOT EXISTS gastos (
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    grupo_id VARCHAR(36) NOT NULL,
    concepto VARCHAR(200) NOT NULL,
    monto_centimos INTEGER NOT NULL,
    fecha DATE NOT NULL,
    pagador_id VARCHAR(36) NOT NULL,
    regla VARCHAR(16) NOT NULL,
    creado_en DATETIME NOT NULL,
    FOREIGN KEY (grupo_id) REFERENCES grupos(id),
    FOREIGN KEY (pagador_id) REFERENCES usuarios(id)
);

CREATE INDEX IF NOT EXISTS ix_gastos_grupo_id
    ON gastos (grupo_id);

CREATE TABLE IF NOT EXISTS partes_gasto (
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    gasto_id VARCHAR(36) NOT NULL,
    usuario_id VARCHAR(36) NOT NULL,
    monto_centimos INTEGER NOT NULL,
    recibio_residuo BOOLEAN NOT NULL,
    CONSTRAINT uq_parte_gasto_usuario UNIQUE (gasto_id, usuario_id),
    FOREIGN KEY (gasto_id) REFERENCES gastos(id),
    FOREIGN KEY (usuario_id) REFERENCES usuarios(id)
);

CREATE INDEX IF NOT EXISTS ix_partes_gasto_gasto_id
    ON partes_gasto (gasto_id);
CREATE INDEX IF NOT EXISTS ix_partes_gasto_usuario_id
    ON partes_gasto (usuario_id);

COMMIT;
