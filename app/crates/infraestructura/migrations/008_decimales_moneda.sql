pragma defer_foreign_keys = ON; 

create table moneda_nueva(
    codigo text primary key, 
    nombre text not null, 
    simbolo text not null,
    decimales integer not null check (decimales >= 0 and decimales <= 8)

); 

--esto es para pasar los datos de una tabla a la otra, como si fueran un respaldo
insert into moneda_nueva (codigo, nombre, simbolo, decimales) 
select codigo, nombre, simbolo, decimales from moneda; 

--borro la tabla anterior moneda y la renombro con el nombre correcto 
drop table moneda; 
alter table moneda_nueva rename to moneda; 