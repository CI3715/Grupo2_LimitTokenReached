use rusqlite::Connection;
use std::sync::Mutex;
use uuid::Uuid;
use aplicacion::puertos::RepositorioMovimientos;
use contrato::movimiento::MovimientoDTO;
use dominio::movimiento::{Movimiento, Ajuste};

pub struct RepositorioMovimientosSqlite {
    pub conexion: Mutex<Connection>,
}

impl RepositorioMovimientosSqlite {
    pub fn new(conn: Connection) -> Self {
        Self {
            conexion: Mutex::new(conn),
        }
    }
}

impl RepositorioMovimientos for RepositorioMovimientosSqlite {
    fn guardar_movimiento(&self, movimiento: &Movimiento) -> Result<MovimientoDTO, String> {
     
        todo!("Implementar guardado de movimiento y proyección de saldo")
    }

    fn guardar_ajuste(&self, ajuste: &Ajuste) -> Result<MovimientoDTO, String> {
        
        
        todo!("Implementar guardado de ajuste compensatorio")
    }

    fn obtener_movimiento_por_id(&self, id: &Uuid) -> Result<Movimiento, String> {
        todo!("Implementar consulta de movimiento original")
    }

    fn obtener_ajustes_de_movimiento(&self, original_id: &Uuid) -> Result<Vec<Ajuste>, String> {
        todo!("Implementar consulta de ajustes previos")
    }
    
    fn obtener_categorias(&self) -> Result<Vec<dominio::categoria::Categoria>, String> {

        todo!("Implementar consulta de categorías en SQLite")
    }
}
