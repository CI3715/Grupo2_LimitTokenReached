use aplicacion::puertos::RepositorioCategorias;
use contrato::categoria::{CategoriaDTO, CrearCategoriaRequest};
use contrato::error_validacion::ErrorValidacionDTO;
use tauri::State;

#[tauri::command]
pub async fn crear_categoria(
    datos: CrearCategoriaRequest,
    repositorio: State<'_, Box<dyn RepositorioCategorias>>,
) -> Result<CategoriaDTO, ErrorValidacionDTO> {
    aplicacion::casos_uso::crear_categoria::ejecutar(datos, repositorio.inner().as_ref()).await
}

#[tauri::command]
pub async fn listar_categorias(
    repositorio: State<'_, Box<dyn RepositorioCategorias>>,
) -> Result<Vec<contrato::categoria::CategoriaDTO>, String> {
    let categorias = repositorio.inner().obtener_categorias()?;

    categorias
        .into_iter()
        .map(|categoria| {
            let tipo = match categoria.tipo {
                dominio::categoria::TipoCategoria::Gasto => {
                    contrato::categoria::TipoCategoriaDTO::Gasto
                }
                dominio::categoria::TipoCategoria::Ingreso => {
                    contrato::categoria::TipoCategoriaDTO::Ingreso
                }
            };

            Ok(contrato::categoria::CategoriaDTO {
                id: categoria.id.to_string(),
                nombre: categoria.nombre,
                tipo,
            })
        })
        .collect()
}
