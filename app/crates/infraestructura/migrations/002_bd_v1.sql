--Esta migracion contiene la base de datos de la version 1
--lo que incluye esta es simplemente la tabla de monedas, la cuenta y la categoria 

create table if not exists moneda  (

    codigo text primary key,
    nombre text not null,
    simbolo text not null,
    decimales integer not null check (decimales >= 0 and decimales <= 10)

);

CREATE TABLE IF NOT EXISTS cuenta (
    id TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    tipo TEXT NOT NULL,
    moneda_codigo TEXT NOT NULL REFERENCES moneda(codigo),
    saldo TEXT NOT NULL
);


--TODO: esta tabla lo que guarda es el tipo de transaccion, su existencia esta en veremos 
CREATE TABLE IF NOT EXISTS categoria (
    id TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    padre_id TEXT REFERENCES categoria(id),
    archivada INTEGER NOT NULL DEFAULT 0
);

insert into moneda (codigo, nombre, simbolo, decimales) values
('VES', 'Bolivar', 'Bs.', 2),
('USD', 'Dolar Estadounidense', '$', 2), 
('EUR', 'Euro', '€', 2), 
('USDT', 'Tether', '₮', 6);