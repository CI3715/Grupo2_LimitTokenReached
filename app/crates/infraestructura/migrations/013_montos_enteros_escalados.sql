PRAGMA defer_foreign_keys = ON;


-- 1. Tabla cuenta: saldo TEXT -> INTEGER (multiplicado por 10^8)
--    Conserva creada_en (006) y activa (012)

CREATE TABLE cuenta_nueva (
    id TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    tipo TEXT NOT NULL,
    moneda_codigo TEXT NOT NULL REFERENCES moneda(codigo),
    saldo INTEGER NOT NULL,
    creada_en TEXT NOT NULL DEFAULT '0001-01-01',
    activa INTEGER NOT NULL DEFAULT 1 CHECK (activa IN (0, 1))
);

INSERT INTO cuenta_nueva (id, nombre, tipo, moneda_codigo, saldo, creada_en, activa)
SELECT
    id,
    nombre,
    tipo,
    moneda_codigo,
    (CASE WHEN substr(trim(saldo), 1, 1) = '-' THEN -1 ELSE 1 END) * (
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(saldo), 1, 1) IN ('-', '+') THEN substr(trim(saldo), 2) ELSE trim(saldo) END, '.') > 0
                THEN substr(
                    CASE WHEN substr(trim(saldo), 1, 1) IN ('-', '+') THEN substr(trim(saldo), 2) ELSE trim(saldo) END,
                    1,
                    instr(CASE WHEN substr(trim(saldo), 1, 1) IN ('-', '+') THEN substr(trim(saldo), 2) ELSE trim(saldo) END, '.') - 1
                )
                ELSE CASE WHEN substr(trim(saldo), 1, 1) IN ('-', '+') THEN substr(trim(saldo), 2) ELSE trim(saldo) END
            END AS INTEGER
        ) * 100000000
        +
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(saldo), 1, 1) IN ('-', '+') THEN substr(trim(saldo), 2) ELSE trim(saldo) END, '.') > 0
                THEN substr(
                    substr(
                        CASE WHEN substr(trim(saldo), 1, 1) IN ('-', '+') THEN substr(trim(saldo), 2) ELSE trim(saldo) END,
                        instr(CASE WHEN substr(trim(saldo), 1, 1) IN ('-', '+') THEN substr(trim(saldo), 2) ELSE trim(saldo) END, '.') + 1
                    ) || '00000000',
                    1,
                    8
                )
                ELSE '00000000'
            END AS INTEGER
        )
    ),
    creada_en,
    activa
FROM cuenta;

DROP TABLE cuenta;
ALTER TABLE cuenta_nueva RENAME TO cuenta;


-- 2. Tabla movimiento: monto TEXT -> INTEGER (multiplicado por 10^8)

CREATE TABLE movimiento_nueva (
    id TEXT PRIMARY KEY,
    cuenta_id TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('Ingreso', 'Gasto')),
    monto INTEGER NOT NULL,
    fecha TEXT NOT NULL,
    categoria_id TEXT NOT NULL,
    nota TEXT,
    FOREIGN KEY (cuenta_id) REFERENCES cuenta(id),
    FOREIGN KEY (categoria_id) REFERENCES categoria(id)
);

INSERT INTO movimiento_nueva (id, cuenta_id, tipo, monto, fecha, categoria_id, nota)
SELECT
    id,
    cuenta_id,
    tipo,
    (CASE WHEN substr(trim(monto), 1, 1) = '-' THEN -1 ELSE 1 END) * (
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(monto), 1, 1) IN ('-', '+') THEN substr(trim(monto), 2) ELSE trim(monto) END, '.') > 0
                THEN substr(
                    CASE WHEN substr(trim(monto), 1, 1) IN ('-', '+') THEN substr(trim(monto), 2) ELSE trim(monto) END,
                    1,
                    instr(CASE WHEN substr(trim(monto), 1, 1) IN ('-', '+') THEN substr(trim(monto), 2) ELSE trim(monto) END, '.') - 1
                )
                ELSE CASE WHEN substr(trim(monto), 1, 1) IN ('-', '+') THEN substr(trim(monto), 2) ELSE trim(monto) END
            END AS INTEGER
        ) * 100000000
        +
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(monto), 1, 1) IN ('-', '+') THEN substr(trim(monto), 2) ELSE trim(monto) END, '.') > 0
                THEN substr(
                    substr(
                        CASE WHEN substr(trim(monto), 1, 1) IN ('-', '+') THEN substr(trim(monto), 2) ELSE trim(monto) END,
                        instr(CASE WHEN substr(trim(monto), 1, 1) IN ('-', '+') THEN substr(trim(monto), 2) ELSE trim(monto) END, '.') + 1
                    ) || '00000000',
                    1,
                    8
                )
                ELSE '00000000'
            END AS INTEGER
        )
    ),
    fecha,
    categoria_id,
    nota
FROM movimiento;

DROP TABLE movimiento;
ALTER TABLE movimiento_nueva RENAME TO movimiento;

CREATE INDEX IF NOT EXISTS idx_movimiento_cuenta_fecha
    ON movimiento (cuenta_id, fecha);

-- ============================================================================
-- 3. Tabla ajuste: variacion_monto TEXT -> INTEGER (multiplicado por 10^8)
--    Conserva fecha (006) y orden (011)

CREATE TABLE ajuste_nueva (
    id TEXT PRIMARY KEY,
    movimiento_original_id TEXT NOT NULL,
    cuenta_id TEXT NOT NULL,
    tipo_original TEXT NOT NULL CHECK (tipo_original IN ('Ingreso', 'Gasto')),
    variacion_monto INTEGER NOT NULL,
    fecha TEXT,
    orden INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (movimiento_original_id) REFERENCES movimiento(id),
    FOREIGN KEY (cuenta_id) REFERENCES cuenta(id)
);

INSERT INTO ajuste_nueva (id, movimiento_original_id, cuenta_id, tipo_original, variacion_monto, fecha, orden)
SELECT
    id,
    movimiento_original_id,
    cuenta_id,
    tipo_original,
    (CASE WHEN substr(trim(variacion_monto), 1, 1) = '-' THEN -1 ELSE 1 END) * (
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(variacion_monto), 1, 1) IN ('-', '+') THEN substr(trim(variacion_monto), 2) ELSE trim(variacion_monto) END, '.') > 0
                THEN substr(
                    CASE WHEN substr(trim(variacion_monto), 1, 1) IN ('-', '+') THEN substr(trim(variacion_monto), 2) ELSE trim(variacion_monto) END,
                    1,
                    instr(CASE WHEN substr(trim(variacion_monto), 1, 1) IN ('-', '+') THEN substr(trim(variacion_monto), 2) ELSE trim(variacion_monto) END, '.') - 1
                )
                ELSE CASE WHEN substr(trim(variacion_monto), 1, 1) IN ('-', '+') THEN substr(trim(variacion_monto), 2) ELSE trim(variacion_monto) END
            END AS INTEGER
        ) * 100000000
        +
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(variacion_monto), 1, 1) IN ('-', '+') THEN substr(trim(variacion_monto), 2) ELSE trim(variacion_monto) END, '.') > 0
                THEN substr(
                    substr(
                        CASE WHEN substr(trim(variacion_monto), 1, 1) IN ('-', '+') THEN substr(trim(variacion_monto), 2) ELSE trim(variacion_monto) END,
                        instr(CASE WHEN substr(trim(variacion_monto), 1, 1) IN ('-', '+') THEN substr(trim(variacion_monto), 2) ELSE trim(variacion_monto) END, '.') + 1
                    ) || '00000000',
                    1,
                    8
                )
                ELSE '00000000'
            END AS INTEGER
        )
    ),
    fecha,
    orden
FROM ajuste;

DROP TABLE ajuste;
ALTER TABLE ajuste_nueva RENAME TO ajuste;

CREATE INDEX IF NOT EXISTS idx_ajuste_cuenta_fecha
    ON ajuste (cuenta_id, fecha);
CREATE INDEX IF NOT EXISTS idx_ajuste_original_orden
    ON ajuste (movimiento_original_id, orden);


-- 4. Tabla partes_grupo_pendientes: monto_parte TEXT -> INTEGER (x 10^8)

CREATE TABLE partes_grupo_pendientes_nueva (
    id_evento TEXT PRIMARY KEY,
    grupo_id TEXT NOT NULL,
    gasto_id TEXT NOT NULL,
    concepto TEXT NOT NULL,
    fecha TEXT NOT NULL,
    moneda TEXT NOT NULL,
    monto_parte INTEGER NOT NULL,
    incluye_redondeo INTEGER NOT NULL CHECK (incluye_redondeo IN (0, 1)),
    posicion_cursor INTEGER NOT NULL,
    recibido_en TEXT NOT NULL
);

INSERT INTO partes_grupo_pendientes_nueva (
    id_evento, grupo_id, gasto_id, concepto, fecha, moneda,
    monto_parte, incluye_redondeo, posicion_cursor, recibido_en
)
SELECT
    id_evento,
    grupo_id,
    gasto_id,
    concepto,
    fecha,
    moneda,
    (CASE WHEN substr(trim(monto_parte), 1, 1) = '-' THEN -1 ELSE 1 END) * (
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(monto_parte), 1, 1) IN ('-', '+') THEN substr(trim(monto_parte), 2) ELSE trim(monto_parte) END, '.') > 0
                THEN substr(
                    CASE WHEN substr(trim(monto_parte), 1, 1) IN ('-', '+') THEN substr(trim(monto_parte), 2) ELSE trim(monto_parte) END,
                    1,
                    instr(CASE WHEN substr(trim(monto_parte), 1, 1) IN ('-', '+') THEN substr(trim(monto_parte), 2) ELSE trim(monto_parte) END, '.') - 1
                )
                ELSE CASE WHEN substr(trim(monto_parte), 1, 1) IN ('-', '+') THEN substr(trim(monto_parte), 2) ELSE trim(monto_parte) END
            END AS INTEGER
        ) * 100000000
        +
        CAST(
            CASE
                WHEN instr(CASE WHEN substr(trim(monto_parte), 1, 1) IN ('-', '+') THEN substr(trim(monto_parte), 2) ELSE trim(monto_parte) END, '.') > 0
                THEN substr(
                    substr(
                        CASE WHEN substr(trim(monto_parte), 1, 1) IN ('-', '+') THEN substr(trim(monto_parte), 2) ELSE trim(monto_parte) END,
                        instr(CASE WHEN substr(trim(monto_parte), 1, 1) IN ('-', '+') THEN substr(trim(monto_parte), 2) ELSE trim(monto_parte) END, '.') + 1
                    ) || '00000000',
                    1,
                    8
                )
                ELSE '00000000'
            END AS INTEGER
        )
    ),
    incluye_redondeo,
    posicion_cursor,
    recibido_en
FROM partes_grupo_pendientes;

DROP TABLE partes_grupo_pendientes;
ALTER TABLE partes_grupo_pendientes_nueva RENAME TO partes_grupo_pendientes;

CREATE INDEX IF NOT EXISTS idx_eventos_grupo_posicion
    ON partes_grupo_pendientes(posicion_cursor);