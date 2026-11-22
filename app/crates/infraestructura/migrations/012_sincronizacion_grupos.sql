CREATE TABLE IF NOT EXISTS sincronizacion_grupo_meta (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cuenta_por_grupo (
    grupo_id TEXT PRIMARY KEY,
    cuenta_id TEXT NOT NULL,
    moneda TEXT NOT NULL,
    actualizada_en TEXT NOT NULL,
    FOREIGN KEY (cuenta_id) REFERENCES cuenta(id)
);

CREATE TABLE IF NOT EXISTS eventos_grupo_aplicados (
    id_evento TEXT PRIMARY KEY,
    movimiento_id TEXT NOT NULL,
    aplicado_en TEXT NOT NULL,
    FOREIGN KEY (movimiento_id) REFERENCES movimiento(id)
);

CREATE TABLE IF NOT EXISTS partes_grupo_pendientes (
    id_evento TEXT PRIMARY KEY,
    grupo_id TEXT NOT NULL,
    gasto_id TEXT NOT NULL,
    concepto TEXT NOT NULL,
    fecha TEXT NOT NULL,
    moneda TEXT NOT NULL,
    monto_parte TEXT NOT NULL,
    incluye_redondeo INTEGER NOT NULL CHECK (incluye_redondeo IN (0, 1)),
    posicion_cursor INTEGER NOT NULL,
    recibido_en TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cola_gastos_grupo (
    idempotency_key TEXT PRIMARY KEY,
    grupo_id TEXT NOT NULL,
    moneda TEXT NOT NULL,
    datos_json TEXT NOT NULL CHECK (json_valid(datos_json)),
    estado TEXT NOT NULL CHECK (estado IN ('pendiente', 'rechazado')),
    motivo TEXT,
    orden INTEGER NOT NULL,
    creado_en TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS movimiento_grupo (
    movimiento_id TEXT PRIMARY KEY,
    id_evento TEXT NOT NULL UNIQUE,
    grupo_id TEXT NOT NULL,
    gasto_id TEXT NOT NULL,
    concepto_grupo TEXT NOT NULL,
    moneda TEXT NOT NULL,
    FOREIGN KEY (movimiento_id) REFERENCES movimiento(id)
);

ALTER TABLE cuenta ADD COLUMN activa INTEGER NOT NULL DEFAULT 1
    CHECK (activa IN (0, 1));

INSERT OR IGNORE INTO categoria (
    id,
    nombre,
    tipo,
    padre_id,
    archivada
) VALUES (
    '00000000-0000-4000-8000-000000000001',
    'Gastos de grupo',
    'Gasto',
    NULL,
    0
);

CREATE INDEX IF NOT EXISTS idx_eventos_grupo_posicion
    ON partes_grupo_pendientes(posicion_cursor);

CREATE INDEX IF NOT EXISTS idx_cola_gastos_orden
    ON cola_gastos_grupo(orden);