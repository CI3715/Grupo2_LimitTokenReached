-- Impide categorías repetidas dentro del mismo tipo.
-- La comparación ignora espacios externos y mayúsculas/minúsculas.
CREATE UNIQUE INDEX IF NOT EXISTS ux_categoria_tipo_nombre
ON categoria (tipo, lower(trim(nombre)));