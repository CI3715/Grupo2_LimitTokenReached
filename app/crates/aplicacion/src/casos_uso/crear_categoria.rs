use crate::puertos::RepositorioCategorias;
use contrato::categoria::{
    CategoriaDTO, CrearCategoriaRequest, TipoCategoriaDTO,
};
use contrato::error_validacion::ErrorValidacionDTO;
use dominio::categoria::{
    crear_categoria as crear_categoria_dominio, TipoCategoria,
};
use dominio::errores::ErrorDominio;
use uuid::Uuid;

/// Crea una categoría después de validar su nombre y tipo
/// contra las categorías existentes.
pub async fn ejecutar(
    datos: CrearCategoriaRequest,
    repositorio: &dyn RepositorioCategorias,
) -> Result<CategoriaDTO, ErrorValidacionDTO> {
    let tipo = match datos.tipo {
        TipoCategoriaDTO::Gasto => TipoCategoria::Gasto,
        TipoCategoriaDTO::Ingreso => TipoCategoria::Ingreso,
    };

    let existentes = repositorio
        .obtener_categorias()
        .map_err(|mensaje| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        })?;

    let categoria = crear_categoria_dominio(
        &existentes,
        Uuid::new_v4(),
        &datos.nombre,
        tipo,
    )
    .map_err(|error| match error {
        ErrorDominio::NombreCategoriaVacio => ErrorValidacionDTO {
            campo: "nombre".to_string(),
            mensaje: "El nombre de la categoría no puede estar vacío.".to_string(),
        },
        ErrorDominio::CategoriaDuplicada => ErrorValidacionDTO {
            campo: "nombre".to_string(),
            mensaje: "Ya existe una categoría con ese nombre para ese tipo."
                .to_string(),
        },
        error => ErrorValidacionDTO {
            campo: error.campo().to_string(),
            mensaje: error.to_string(),
        },
    })?;

    repositorio
        .guardar_categoria(&categoria)
        .map_err(|mensaje| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        })
}