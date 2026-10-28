use crate::puertos::{Reloj, RepositorioMonedas, RepositorioTasas};
use chrono::NaiveDate;
use contrato::{
    error_validacion::ErrorValidacionDTO,
    tasa::{RegistrarTasaRequest, TasaDTO},
};
use rust_decimal::Decimal;
use std::str::FromStr;

pub async fn ejecutar(
    datos: RegistrarTasaRequest,
    repo_tasas: &dyn RepositorioTasas,
    repo_monedas: &dyn RepositorioMonedas,
    reloj: &dyn Reloj,
) -> Result<TasaDTO, ErrorValidacionDTO> {
    let moneda_origen = datos.moneda_origen.trim().to_uppercase();
    let moneda_destino = datos.moneda_destino.trim().to_uppercase();
    if moneda_origen == moneda_destino {
        return Err(error("moneda_destino", "Las monedas deben ser diferentes."));
    }

    let monedas = repo_monedas.obtener_monedas().map_err(error_general)?;
    for codigo in [&moneda_origen, &moneda_destino] {
        if !monedas.iter().any(|moneda| moneda.codigo == *codigo) {
            return Err(error(
                "moneda",
                &format!("La moneda '{codigo}' no existe en el catálogo."),
            ));
        }
    }

    let valor = Decimal::from_str(datos.valor.trim())
        .map_err(|_| error("valor", "El valor debe ser un número decimal válido."))?;
    if valor <= Decimal::ZERO {
        return Err(error(
            "valor",
            "El valor de la tasa debe ser mayor que cero.",
        ));
    }

    if valor.normalize().scale() > 8 {
        return Err(error("valor", "La tasa no puede tener más de 8 decimales."));
    }

    let fecha = NaiveDate::parse_from_str(&datos.fecha, "%Y-%m-%d")
        .map_err(|_| error("fecha", "Formato inválido. Use AAAA-MM-DD."))?;
    let hoy = reloj
        .ahora()
        .get(..10)
        .and_then(|fecha| NaiveDate::parse_from_str(fecha, "%Y-%m-%d").ok())
        .ok_or_else(|| error("fecha", "No se pudo obtener la fecha actual."))?;
    if fecha > hoy {
        return Err(error("fecha", "La fecha de la tasa no puede ser futura."));
    }

    let fuente = datos.fuente.trim();
    if fuente.is_empty() {
        return Err(error("fuente", "La fuente no puede estar vacía."));
    }

    let datos = RegistrarTasaRequest {
        moneda_origen,
        moneda_destino,
        valor: datos.valor.trim().to_string(),
        fecha: fecha.to_string(),
        fuente: fuente.to_string(),
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
