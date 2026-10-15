
--Esta es la tabla donde se registran los eventos que sucedan en el sistema
create table if not exists events (
    seq integer primary key autoincrement,
    event_type text not null unique,
    tipo text not null,
    registered_at text not null,
    origin text not null,
    data text not null check (json_valid(data))

);


--Estos trigger evita que se actualicen eventos, ya que son inmutables (INV-03)
create trigger if not exists events_without_update
before update on events 
begin select raise  (abort, 'Los eventos deben ser inmutables'); end ; 

create trigger if not exists events_without_delete
before delete on events raise (abort, 'Los eventos deben ser inmutables'); end ;

create table if not exists meta (clave text primary key, valor text not null);
