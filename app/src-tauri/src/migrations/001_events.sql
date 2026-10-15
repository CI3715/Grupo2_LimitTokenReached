
--Esta es la tabla donde se registran los eventos que sucedan en el sistema
create table if not exists eventos (
    seq integer primary key autoincrement,
    id text not null unique,
    tipo text not null,
    registrado_en text not null,
    dispositivo text not null,
    datos text not null check (json_valid(datos))

);


--Estos trigger evita que se actualicen eventos, ya que son inmutables (INV-03)
create trigger if not exists events_without_update
before update on eventos 
begin 
    select raise  (abort, 'Los eventos deben ser inmutables') ;
end; 

create trigger if not exists events_without_delete
before delete on eventos
begin 
    select raise (abort, 'Los eventos deben ser inmutables');
end;

create table if not exists meta (clave text primary key, valor text not null);
