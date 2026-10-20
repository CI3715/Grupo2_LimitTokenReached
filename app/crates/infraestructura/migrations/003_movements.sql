--Esta tabla se crea con el fin de registrar las categorias de movimientos

alter table categoria add column tipo text not null default 'Gasto'
    check (tipo in ('Gasto', 'Ingreso'));

--NOTE: El saldo de la cuenta se va actualizando sumando el saldo inicial de la cuenta
--con los movimientos actuales, de manera que sumemos todo
create table if not exists movimiento (
    id integer primary key autoincrement, 
    nombre varchar not null, 
    tipo integer not null references categoria(id),
    cuenta integer references cuenta(id), 
    monto text not null 
    
);

--inseerciones de  en la tabla categoria
insert into categoria (id, tipo, nombre) values 
(1, "Gasto","Alimentacion" ),
(2, "Gasto","Transporte" ),
(3, "Gasto","Servicios" ),
(4, "Gasto","Salud" ),
(5, "Gasto", "Educacion" ),
(6, "Gasto", "Entretenimiento"),
(7, "Gasto","Otros Gastos" ),
(8, "Ingreso", "Salario"), 
(9, "Ingreso", "Ventas" ),
(10, "Ingreso", "Regalos"),
(11, "Ingreso", "Otros Ingresos");
