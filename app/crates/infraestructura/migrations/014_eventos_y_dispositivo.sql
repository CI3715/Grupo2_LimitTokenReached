CREATE TABLE IF NOT EXISTS meta (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_eventos_orden_determinista
    ON eventos (registrado_en ASC, dispositivo ASC, id ASC);