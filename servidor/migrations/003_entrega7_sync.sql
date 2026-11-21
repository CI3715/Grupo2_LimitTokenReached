PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

CREATE TABLE IF NOT EXISTS cambios_gasto (
    id_evento VARCHAR(36) NOT NULL PRIMARY KEY,
    usuario_id VARCHAR(36) NOT NULL,
    posicion INTEGER NOT NULL,
    tipo VARCHAR(60) NOT NULL,
    grupo_id VARCHAR(36) NOT NULL,
    gasto_id VARCHAR(36) NOT NULL,
    concepto VARCHAR(200) NOT NULL,
    fecha DATE NOT NULL,
    moneda VARCHAR(3) NOT NULL,
    monto_centimos INTEGER NOT NULL,
    incluye_redondeo BOOLEAN NOT NULL,
    CONSTRAINT uq_cambio_usuario_posicion UNIQUE (usuario_id, posicion),
    FOREIGN KEY (usuario_id) REFERENCES usuarios(id),
    FOREIGN KEY (grupo_id) REFERENCES grupos(id),
    FOREIGN KEY (gasto_id) REFERENCES gastos(id)
);

CREATE INDEX IF NOT EXISTS ix_cambios_gasto_usuario_id
    ON cambios_gasto (usuario_id);
CREATE INDEX IF NOT EXISTS ix_cambios_gasto_usuario_posicion
    ON cambios_gasto (usuario_id, posicion);

CREATE TABLE IF NOT EXISTS operaciones_idempotentes (
    clave VARCHAR(200) NOT NULL PRIMARY KEY,
    usuario_id VARCHAR(36) NOT NULL,
    ruta VARCHAR(300) NOT NULL,
    datos_hash VARCHAR(64) NOT NULL,
    respuesta_json TEXT NOT NULL,
    creada_en DATETIME NOT NULL,
    FOREIGN KEY (usuario_id) REFERENCES usuarios(id)
);

COMMIT;
