UPDATE moneda
SET decimales = 2
WHERE codigo = 'USDT'
  AND decimales <> 2;