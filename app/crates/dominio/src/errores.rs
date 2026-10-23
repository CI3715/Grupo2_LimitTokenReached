use thiserror::Error;

/// Errores de validación del dominio. Cada uno sabe qué campo falló,
/// para que la interfaz pueda señalarlo sin mostrar trazas técnicas.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorDominio {
    #[error("El monto debe ser mayor que cero.")]
    MontoNoPositivo,

    #[error("El monto no puede tener más de {maximo} decimales para {moneda}.")]
    DecimalesExcedidos { moneda: String, maximo: u32 },

    #[error("Debes elegir una categoría.")]
    CategoriaVacia,

    #[error("La categoría elegida no existe.")]
    CategoriaInexistente,

    #[error("La categoría no corresponde al tipo de movimiento.")]
    CategoriaTipoIncorrecto,

    #[error("La fecha no puede ser futura.")]
    FechaFutura,

    #[error("El nombre de la categoría no puede estar vacío.")]
    NombreCategoriaVacio,

    #[error("Ya existe una categoría con ese nombre en ese tipo.")]
    CategoriaDuplicada,

    #[error("El monto correcto no puede ser negativo.")]
    MontoCorreccionNegativo,

    #[error("El monto correcto es igual al valor actual del movimiento.")]
    CorreccionSinCambio,
}

impl ErrorDominio {
    /// Nombre del campo del formulario que provocó el error.
    pub fn campo(&self) -> &'static str {
        match self {
            Self::MontoNoPositivo | Self::DecimalesExcedidos { .. } => "monto",
            Self::CategoriaVacia | Self::CategoriaInexistente | Self::CategoriaTipoIncorrecto => {
                "categoria"
            }
            Self::FechaFutura => "fecha",
            Self::NombreCategoriaVacio | Self::CategoriaDuplicada => "nombre",
            Self::MontoCorreccionNegativo | Self::CorreccionSinCambio => "monto_correcto",
        }
    }
}
