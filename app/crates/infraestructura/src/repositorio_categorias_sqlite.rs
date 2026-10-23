use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde_json::json;
use uuid::Uuid;

use aplicacion::puertos::RepositorioCategorias;
use contrato::categoria::{CategoriaDTO, TipoCategoriaDTO};
use dominio::categoria::{Categoria, TipoCategoria};

use crate::bd;

pub struct RepositorioCategoriasSqlite {
    pub conexion: Mutex<Connection>,
}

impl RepositorioCategoriasSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

fn tipo_a_texto(tipo: TipoCategoria) -> &'static str {
    match tipo {
        TipoCategoria::Gasto => "Gasto",
        TipoCategoria::Ingreso => "Ingreso",
    }
}

fn texto_a_tipo(valor: &str) -> Result<TipoCategoria, String> {
    match valor {
        "Gasto" => Ok(TipoCategoria::Gasto),
        "Ingreso" => Ok(TipoCategoria::Ingreso),
        otro => Err(format!("Tipo de categoría inválido: {otro}")),
    }
}

fn tipo_a_dto(tipo: TipoCategoria) -> TipoCategoriaDTO {
    match tipo {
        TipoCategoria::Gasto => TipoCategoriaDTO::Gasto,
        TipoCategoria::Ingreso => TipoCategoriaDTO::Ingreso,
    }
}

impl RepositorioCategorias for RepositorioCategoriasSqlite {
    fn obtener_categorias(&self) -> Result<Vec<Categoria>, String> {
        let conexion = self.conexion.lock().map_err(|e| e.to_string())?;

        let mut consulta = conexion
            .prepare(
                "
                SELECT id, nombre, tipo
                FROM categoria
                WHERE archivada = 0
                ORDER BY tipo, nombre
                ",
            )
            .map_err(|e| format!("No se pudieron consultar las categorías: {e}"))?;

        let filas = consulta
            .query_map([], |fila| {
                let id: String = fila.get(0)?;
                let nombre: String = fila.get(1)?;
                let tipo: String = fila.get(2)?;

                Ok((id, nombre, tipo))
            })
            .map_err(|e| format!("No se pudieron leer las categorías: {e}"))?;

        let mut categorias = Vec::new();

        for fila in filas {
            let (id, nombre, tipo) =
                fila.map_err(|e| format!("No se pudo leer una categoría: {e}"))?;

            categorias.push(Categoria {
                id: Uuid::parse_str(&id)
                    .map_err(|e| format!("El id de categoría no es válido: {e}"))?,
                nombre,
                tipo: texto_a_tipo(&tipo)?,
            });
        }

        Ok(categorias)
    }

    fn guardar_categoria(
        &self,
        categoria: &Categoria,
    ) -> Result<CategoriaDTO, String> {
        let mut conexion = self.conexion.lock().map_err(|e| e.to_string())?;
        let transaccion = conexion
            .transaction()
            .map_err(|e| format!("No se pudo iniciar la transacción: {e}"))?;

        let datos_evento = json!({
            "id": categoria.id.to_string(),
            "nombre": categoria.nombre,
            "tipo": tipo_a_texto(categoria.tipo),
        });

        bd::insertar_evento(&transaccion, "CategoriaCreada", &datos_evento)
            .map_err(|e| format!("No se pudo registrar el evento: {e}"))?;

        transaccion
            .execute(
                "
                INSERT INTO categoria (id, nombre, tipo)
                VALUES (?1, ?2, ?3)
                ",
                params![
                    categoria.id.to_string(),
                    categoria.nombre,
                    tipo_a_texto(categoria.tipo),
                ],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    format!(
                        "Ya existe una categoría con el mismo nombre para el tipo {}.",
                        tipo_a_texto(categoria.tipo)
                    )
                } else {
                    format!("No se pudo guardar la categoría: {e}")
                }
            })?;

        transaccion
            .commit()
            .map_err(|e| format!("No se pudo confirmar la categoría: {e}"))?;

        Ok(CategoriaDTO {
            id: categoria.id.to_string(),
            nombre: categoria.nombre.clone(),
            tipo: tipo_a_dto(categoria.tipo),
        })
    }
}