use crate::puertos::{Reloj, RepositorioMonedas, RepositorioTasas};
use contrato::{
    error_validacion::ErrorValidacionDTO,
    tasa::{RegistrarTasaRequest, TasaDTO},
};
use chrono::NaiveDate;
use dominio::tasas::TasaValidada;

pub async fn ejecutar(
    datos: RegistrarTasaRequest,
    repo_tasas: &dyn RepositorioTasas,
    repo_monedas: &dyn RepositorioMonedas,
    reloj: &dyn Reloj,
) -> Result<TasaDTO, ErrorValidacionDTO> {
    let monedas = repo_monedas.obtener_monedas().map_err(error_general)?;
    let hoy = reloj
        .ahora()
        .get(..10)
        .and_then(|fecha| NaiveDate::parse_from_str(fecha, "%Y-%m-%d").ok())
        .ok_or_else(|| error("fecha", "No se pudo obtener la fecha actual."))?;
    let codigos_catalogo = monedas
        .iter()
        .map(|moneda| moneda.codigo.clone())
        .collect::<Vec<_>>();
    let tasa = TasaValidada::validar(
        &datos.moneda_origen,
        &datos.moneda_destino,
        &datos.valor,
        &datos.fecha,
        &datos.fuente,
        &codigos_catalogo,
        hoy,
    )
    .map_err(|validation_error| {
        error(validation_error.campo(), &validation_error.to_string())
    })?;

    let datos = RegistrarTasaRequest {
        moneda_origen: tasa.moneda_origen,
        moneda_destino: tasa.moneda_destino,
        valor: datos.valor.trim().to_string(),
        fecha: tasa.fecha_aplicable.to_string(),
        fuente: tasa.fuente,
    };
    repo_tasas.guardar_tasa(&datos).map_err(error_general)
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
