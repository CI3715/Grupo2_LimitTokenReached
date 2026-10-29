use crate::puertos::RepositorioMonedas;
use ::contrato::{
    error_validacion::ErrorValidacionDTO,
    moneda::{CODIGOS_MONEDAS_CATALOGO, EliminarMonedaRequest},
};

pub async fn ejecutar(
    datos: EliminarMonedaRequest,
    repo: &dyn RepositorioMonedas,
) -> Result<(), ErrorValidacionDTO> {
    let codigo = datos.codigo.trim();

    if codigo.is_empty() {
        return Err(error(
            "codigo",
            "El código de la moneda no puede estar vacío.",
        ));
    }

    if CODIGOS_MONEDAS_CATALOGO.contains(&codigo) {
        return Err(error(
            "codigo",
            "Las monedas de fábrica no se pueden eliminar.",
        ));
    }

    repo.eliminar_moneda(codigo)
        .map_err(|mensaje| error("general", &mensaje))
}

fn error(campo: &str, mensaje: &str) -> ErrorValidacionDTO {
    ErrorValidacionDTO {
        campo: campo.to_string(),
        mensaje: mensaje.to_string(),
    }
}
