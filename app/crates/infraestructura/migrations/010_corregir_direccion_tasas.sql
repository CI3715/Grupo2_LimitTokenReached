UPDATE tasas
SET moneda_entrada = 'USD',
    moneda_salida = 'VES'
WHERE tasa_id = 1
  AND moneda_entrada = 'VES'
  AND moneda_salida = 'USD'
  AND valor = '887.96';

UPDATE tasas
SET moneda_entrada = 'USDT',
    moneda_salida = 'VES'
WHERE tasa_id = 3
  AND moneda_entrada = 'VES'
  AND moneda_salida = 'USDT'
  AND valor = '1050.26';