use serde::{Deserialize, Serialize};

/// Tipo de categoría disponible para clasificar movimientos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TipoCategoriaDTO {
    Gasto,
    Ingreso,
}

/// Datos enviados por el frontend para crear una categoría.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrearCategoriaRequest {
    pub nombre: String,
    pub tipo: TipoCategoriaDTO,
}

/// Categoría devuelta al frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoriaDTO {
    pub id: String,
    pub nombre: String,
    pub tipo: TipoCategoriaDTO,
}
