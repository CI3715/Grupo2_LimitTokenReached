use crate::puertos::{RepositorioMonedas, RepositorioPreferencias};
use contrato::{
    error_validacion::ErrorValidacionDTO,
    moneda::CambiarMonedaBaseRequest,
};

pub async fn ejecutar(
    datos: CambiarMonedaBaseRequest,
    repo_preferencias: &dyn RepositorioPreferencias,
    repo_monedas: &dyn RepositorioMonedas,
) -> Result<(), ErrorValidacionDTO> {
    let codigo = datos.codigo.trim().to_uppercase();
    let monedas = repo_monedas.obtener_monedas().map_err(error_general)?;
    if !monedas.iter().any(|moneda| moneda.codigo == codigo) {
        return Err(ErrorValidacionDTO {
            campo: "codigo".to_string(),
            mensaje: format!("La moneda '{codigo}' no existe en el catálogo."),
        });
    }

    repo_preferencias
        .guardar_moneda_base(&codigo)
        .map_err(error_general)
}

fn error_general(mensaje: String) -> ErrorValidacionDTO {
    ErrorValidacionDTO {
        campo: "general".to_string(),
        mensaje,
    }
}
