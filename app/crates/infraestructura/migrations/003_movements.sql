--Esta tabla se crea con el fin de registrar las categorias de movimientos

ALTER TABLE categoria ADD COLUMN tipo TEXT NOT NULL DEFAULT 'Gasto'
    CHECK (tipo IN ('Gasto', 'Ingreso'));

--NOTE: El saldo de la cuenta se va actualizando sumando el saldo inicial de la cuenta
--con los movimientos actuales, de manera que sumemos todo
CREATE TABLE IF NOT EXISTS movimiento (
    id TEXT PRIMARY KEY,
    cuenta_id TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('Ingreso', 'Gasto')),
    monto TEXT NOT NULL,
    fecha TEXT NOT NULL,
    categoria_id TEXT NOT NULL,
    nota TEXT,
    FOREIGN KEY (cuenta_id) REFERENCES cuenta(id),
    FOREIGN KEY (categoria_id) REFERENCES categoria(id)
);


CREATE TABLE IF NOT EXISTS ajuste (
    id TEXT PRIMARY KEY,
    movimiento_original_id TEXT NOT NULL,
    cuenta_id TEXT NOT NULL,
    tipo_original TEXT NOT NULL CHECK (tipo_original IN ('Ingreso', 'Gasto')),
    variacion_monto TEXT NOT NULL,
    FOREIGN KEY (movimiento_original_id) REFERENCES movimiento(id),
    FOREIGN KEY (cuenta_id) REFERENCES cuenta(id)
);

--inseerciones de  en la tabla categoria
-- Los IDs deben ser UUID compatibles con: Uuid::from_u128(i as u128 + 1)
-- Por ejemplo:
INSERT INTO categoria (id, tipo, nombre) VALUES
('00000000-0000-0000-0000-000000000001', 'Gasto', 'Alimentación'),
('00000000-0000-0000-0000-000000000002', 'Gasto', 'Transporte'),
('00000000-0000-0000-0000-000000000003', 'Gasto', 'Servicios'),
('00000000-0000-0000-0000-000000000004', 'Gasto', 'Salud'),
('00000000-0000-0000-0000-000000000005', 'Gasto', 'Educación'),
('00000000-0000-0000-0000-000000000006', 'Gasto', 'Entretenimiento'),
('00000000-0000-0000-0000-000000000007', 'Gasto', 'Otros gastos'),
('00000000-0000-0000-0000-000000000008', 'Ingreso', 'Salario'),
('00000000-0000-0000-0000-000000000009', 'Ingreso', 'Ventas'),
('00000000-0000-0000-0000-00000000000a', 'Ingreso', 'Regalos'),
('00000000-0000-0000-0000-00000000000b', 'Ingreso', 'Otros ingresos');