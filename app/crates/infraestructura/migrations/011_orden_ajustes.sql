ALTER TABLE ajuste ADD COLUMN orden INTEGER NOT NULL DEFAULT 0;

UPDATE ajuste
SET orden = (
    SELECT COUNT(*)
    FROM ajuste anterior
    WHERE anterior.movimiento_original_id = ajuste.movimiento_original_id
      AND anterior.rowid <= ajuste.rowid
);

CREATE INDEX IF NOT EXISTS idx_ajuste_original_orden
    ON ajuste (movimiento_original_id, orden);
