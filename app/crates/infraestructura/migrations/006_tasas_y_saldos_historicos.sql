ALTER TABLE cuenta ADD COLUMN creada_en TEXT NOT NULL DEFAULT '0001-01-01';
ALTER TABLE ajuste ADD COLUMN fecha TEXT;

UPDATE ajuste
SET fecha = date('now')
WHERE fecha IS NULL;

UPDATE tasas
SET fecha = substr(fecha, 7, 4) || '-' || substr(fecha, 4, 2) || '-' || substr(fecha, 1, 2)
WHERE fecha LIKE '__/__/____';

ALTER TABLE tasas RENAME TO tasas_con_precision_anterior;
CREATE TABLE tasas (
    tasa_id INTEGER PRIMARY KEY NOT NULL,
    moneda_entrada TEXT NOT NULL REFERENCES moneda(codigo),
    moneda_salida TEXT NOT NULL REFERENCES moneda(codigo),
    valor TEXT NOT NULL,
    fecha TEXT NOT NULL,
    fuente TEXT NOT NULL
);
INSERT INTO tasas (tasa_id, moneda_entrada, moneda_salida, valor, fecha, fuente)
SELECT tasa_id, moneda_entrada, moneda_salida, CAST(valor AS TEXT), fecha, fuente
FROM tasas_con_precision_anterior;
DROP TABLE tasas_con_precision_anterior;

CREATE INDEX IF NOT EXISTS idx_tasas_par_fecha
    ON tasas (moneda_entrada, moneda_salida, fecha);
CREATE INDEX IF NOT EXISTS idx_movimiento_cuenta_fecha
    ON movimiento (cuenta_id, fecha);
CREATE INDEX IF NOT EXISTS idx_ajuste_cuenta_fecha
    ON ajuste (cuenta_id, fecha);
