--Esta migracion contiene la base de datos de la version 1
--lo que incluye esta es simplemente la tabla de monedas, la cuenta y la categoria 

create table if not exists moneda  (

    codigo text primary key,
    nombre text not null,
    simbolo text not null,
    decimales integer not null check (decimales >= 0 and decimales <= 10)

);

create table if not exists cuenta  (
    id integer primary key autoincrement,
    nombre text not null,
    tipo text not null,
    moneda_codigo text not null references moneda(codigo),
    saldo real not null check (saldo >= 0)
); 


--TODO: esta tabla lo que guarda es el tipo de transaccion, su existencia esta en veremos 
create table categoria  (
    id text primary key,
    nombre text not null,
    padre_id text references categoria(id), 
    archivada integer not null default 0
    
);

insert into moneda (codigo, nombre, simbolo, decimales) values
('VES', 'Bolivar', 'Bs.', 2),
('USD', 'Dolar Estadounidense', '$', 2), 
('EUR', 'Euro', '€', 2), 
('USDT', 'Tether', '₮', 6);