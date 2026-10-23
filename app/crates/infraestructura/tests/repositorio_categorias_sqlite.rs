use rusqlite::Connection;

use aplicacion::puertos::RepositorioCategorias;
use dominio::categoria::{Categoria, TipoCategoria};
use infraestructura::bd;
use infraestructura::RepositorioCategoriasSqlite;
use uuid::Uuid;

fn crear_repositorio() -> RepositorioCategoriasSqlite {
    let mut conexion =
        Connection::open_in_memory().expect("No se pudo crear SQLite en memoria");

    bd::migrar(&mut conexion).expect("No se pudieron ejecutar las migraciones");

    RepositorioCategoriasSqlite::new(conexion)
}

#[test]
fn guarda_categoria_nueva() {
    let repositorio = crear_repositorio();

    let categoria = Categoria {
        id: Uuid::from_u128(5000),
        nombre: "Mascotas".to_string(),
        tipo: TipoCategoria::Gasto,
    };

    let dto = repositorio
        .guardar_categoria(&categoria)
        .expect("La categoría debería guardarse");

    assert_eq!(dto.id, categoria.id.to_string());
    assert_eq!(dto.nombre, "Mascotas");

    let categorias = repositorio
        .obtener_categorias()
        .expect("Las categorías deberían recuperarse");

    assert!(categorias.iter().any(|item| {
        item.id == categoria.id && item.nombre == "Mascotas"
    }));
}

#[test]
fn sqlite_rechaza_mismo_nombre_ignorando_mayusculas_en_mismo_tipo() {
    let repositorio = crear_repositorio();

    let primera = Categoria {
        id: Uuid::from_u128(5001),
        nombre: "Mascotas".to_string(),
        tipo: TipoCategoria::Gasto,
    };

    let segunda = Categoria {
        id: Uuid::from_u128(5002),
        nombre: "mascotas".to_string(),
        tipo: TipoCategoria::Gasto,
    };

    repositorio
        .guardar_categoria(&primera)
        .expect("La primera categoría debería guardarse");

    let resultado = repositorio.guardar_categoria(&segunda);

    assert!(resultado.is_err());
}

#[test]
fn permite_mismo_nombre_en_tipos_diferentes() {
    let repositorio = crear_repositorio();

    let gasto = Categoria {
        id: Uuid::from_u128(5003),
        nombre: "Premio".to_string(),
        tipo: TipoCategoria::Gasto,
    };

    let ingreso = Categoria {
        id: Uuid::from_u128(5004),
        nombre: "premio".to_string(),
        tipo: TipoCategoria::Ingreso,
    };

    repositorio
        .guardar_categoria(&gasto)
        .expect("La categoría de gasto debería guardarse");

    repositorio
        .guardar_categoria(&ingreso)
        .expect("La categoría de ingreso debería guardarse");
}