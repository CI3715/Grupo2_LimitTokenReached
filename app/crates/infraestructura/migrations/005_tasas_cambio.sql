--tabla donde guardo las tasas 

create table if not exists tasas (
    tasa_id integer primary key not null, 
    moneda_entrada text not null references moneda (codigo),
    moneda_salida text not null references moneda(codigo),
    valor text not null,
    fecha text not null, 
    fuente text not null
);

insert into tasas (tasa_id, moneda_entrada, moneda_salida, valor, fecha, fuente) values
(1, "VES", "USD", 887.96, "27/10/2026", "DolarToday"), 
(2, "EUR", "VES", 980.56, "27/10/2026", "Banco Central de Venezuela"), 
(3, "VES", "USDT", 1050.26, "27/10/2026", "Binance");