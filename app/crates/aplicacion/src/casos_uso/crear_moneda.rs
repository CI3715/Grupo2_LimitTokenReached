use crate::puertos::RepositorioMonedas;
use contrato::{
    error_validacion::ErrorValidacionDTO,
    moneda::{CrearMonedaRequest, MonedaDTO},
};

const MONEDAS_DE_FABRICA: [&str; 4] = ["VES", "USD", "EUR", "USDT"];

pub async fn ejecutar(
    datos: CrearMonedaRequest,
    repo_monedas: &dyn RepositorioMonedas,
) -> Result<MonedaDTO, ErrorValidacionDTO> {
    let codigo = datos.codigo.trim().to_string();
    let nombre = datos.nombre.trim().to_string();
    let simbolo = datos.simbolo.trim().to_string();

    validar_codigo(&codigo)?;
    validar_nombre(&nombre)?;
    validar_simbolo(&simbolo)?;

    if datos.decimales > 8 {
        return Err(error(
            "decimales",
            "La moneda no puede tener más de 8 decimales.",
        ));
    }

    if MONEDAS_DE_FABRICA.contains(&codigo.as_str()) {
        return Err(error(
            "codigo",
            "Las monedas de fábrica no se pueden volver a crear.",
        ));
    }

    let monedas = repo_monedas
        .obtener_monedas()
        .map_err(error_general)?;

    if monedas.iter().any(|moneda| moneda.codigo == codigo) {
        return Err(error(
            "codigo",
            &format!("Ya existe una moneda con el código '{codigo}'."),
        ));
    }

    let request = CrearMonedaRequest {
        codigo,
        nombre,
        simbolo,
        decimales: datos.decimales,
    };

    repo_monedas
        .guardar_moneda(&request)
        .map_err(error_general)
}

fn validar_codigo(codigo: &str) -> Result<(), ErrorValidacionDTO> {
    if !(3..=5).contains(&codigo.len()) {
        return Err(error(
            "codigo",
            "El código debe tener entre 3 y 5 letras mayúsculas.",
        ));
    }

    if !codigo
        .bytes()
        .all(|caracter| caracter.is_ascii_uppercase())
    {
        return Err(error(
            "codigo",
            "El código solo puede contener letras mayúsculas.",
        ));
    }

    Ok(())
}

fn validar_nombre(nombre: &str) -> Result<(), ErrorValidacionDTO> {
    if nombre.is_empty() {
        return Err(error(
            "nombre",
            "El nombre de la moneda no puede estar vacío.",
        ));
    }

    Ok(())
}

fn validar_simbolo(simbolo: &str) -> Result<(), ErrorValidacionDTO> {
    if simbolo.is_empty() {
        return Err(error(
            "simbolo",
            "El símbolo de la moneda no puede estar vacío.",
        ));
    }

    Ok(())
}

fn error(campo: &str, mensaje: &str) -> ErrorValidacionDTO {
    ErrorValidacionDTO {
        campo: campo.to_string(),
        mensaje: mensaje.to_string(),
    }
}

fn error_general(mensaje: String) -> ErrorValidacionDTO {
    error("general", &mensaje)
}