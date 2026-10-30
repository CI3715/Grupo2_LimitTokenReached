use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasaValidada {
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: Decimal,
    pub fecha_aplicable: NaiveDate,
    pub fuente: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorValidacionTasa {
    #[error("Las monedas deben ser diferentes.")]
    MonedasIguales,
    #[error("La moneda '{0}' no existe en el catálogo.")]
    MonedaNoCatalogada(String),
    #[error("El valor debe ser un número decimal válido.")]
    ValorInvalido,
    #[error("El valor de la tasa debe ser mayor que cero.")]
    ValorNoPositivo,
    #[error("La tasa no puede tener más de 8 decimales.")]
    DecimalesExcedidos,
    #[error("La fecha es requerida.")]
    FechaVacia,
    #[error("Formato inválido. Use AAAA-MM-DD.")]
    FechaInvalida,
    #[error("La fecha de la tasa no puede ser futura.")]
    FechaFutura,
    #[error("La fuente no puede estar vacía.")]
    FuenteVacia,
}

impl ErrorValidacionTasa {
    pub fn campo(&self) -> &'static str {
        match self {
            Self::MonedasIguales => "moneda_destino",
            Self::MonedaNoCatalogada(_) => "moneda",
            Self::ValorInvalido | Self::ValorNoPositivo | Self::DecimalesExcedidos => "valor",
            Self::FechaVacia | Self::FechaInvalida | Self::FechaFutura => "fecha",
            Self::FuenteVacia => "fuente",
        }
    }
}

impl TasaValidada {
    pub fn validar(
        moneda_origen: &str,
        moneda_destino: &str,
        valor: &str,
        fecha: &str,
        fuente: &str,
        codigos_catalogo: &[String],
        hoy: NaiveDate,
    ) -> Result<Self, ErrorValidacionTasa> {
        let moneda_origen = moneda_origen.trim().to_uppercase();
        let moneda_destino = moneda_destino.trim().to_uppercase();

        if moneda_origen == moneda_destino {
            return Err(ErrorValidacionTasa::MonedasIguales);
        }

        for codigo in [&moneda_origen, &moneda_destino] {
            if !codigos_catalogo.iter().any(|disponible| disponible == codigo) {
                return Err(ErrorValidacionTasa::MonedaNoCatalogada(codigo.clone()));
            }
        }

        let valor =
            Decimal::from_str(valor.trim()).map_err(|_| ErrorValidacionTasa::ValorInvalido)?;
        if valor <= Decimal::ZERO {
            return Err(ErrorValidacionTasa::ValorNoPositivo);
        }
        if valor.normalize().scale() > 8 {
            return Err(ErrorValidacionTasa::DecimalesExcedidos);
        }

        let fecha = fecha.trim();
        if fecha.is_empty() {
            return Err(ErrorValidacionTasa::FechaVacia);
        }
        let fecha_aplicable = NaiveDate::parse_from_str(fecha, "%Y-%m-%d")
            .map_err(|_| ErrorValidacionTasa::FechaInvalida)?;
        if fecha_aplicable > hoy {
            return Err(ErrorValidacionTasa::FechaFutura);
        }

        let fuente = fuente.trim();
        if fuente.is_empty() {
            return Err(ErrorValidacionTasa::FuenteVacia);
        }

        Ok(Self {
            moneda_origen,
            moneda_destino,
            valor,
            fecha_aplicable,
            fuente: fuente.to_string(),
        })
    }
}

/// Representa el registro de una nueva tasa de cambio.
/// Almacena el valor exacto con Decimal y fechas nativas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasaRegistrada {
    pub id: Uuid,
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: Decimal,
    pub fecha_aplicable: NaiveDate,
    pub fuente: String,
    pub registrada_en: DateTime<Utc>,
}

/// Representa la eliminación lógica (anulación) de una tasa registrada previamente.
/// En Event Sourcing no hacemos DELETE, emitimos este evento para invalidarla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasaEliminada {
    pub tasa_id: Uuid,
    pub fecha_eliminacion: DateTime<Utc>,
}

/// Representa la creación de una nueva moneda en el catálogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonedaCreada {
    pub codigo: String,  // Ej: "VES", "USD"
    pub nombre: String,  // Ej: "Bolívar", "Dólar"
    pub simbolo: String, // Ej: "Bs.", "$"
    pub decimales: u32,
    pub creada_en: DateTime<Utc>,
}

/// Representa un cambio en la configuración global de la moneda base del usuario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonedaBaseCambiada {
    pub nuevo_codigo: String,
    pub cambiada_en: DateTime<Utc>,
}

/// Enum unificado que agrupa todos los eventos del dominio patrimonial.
/// Esto permite que el Repositorio reciba un `EventoPatrimonio` genérico
/// y facilite la serialización/deserialización para la tabla de SQLite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventoPatrimonio {
    TasaRegistrada(TasaRegistrada),
    TasaEliminada(TasaEliminada),
    MonedaCreada(MonedaCreada),
    MonedaBaseCambiada(MonedaBaseCambiada),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogo() -> Vec<String> {
        vec!["USD".to_string(), "VES".to_string()]
    }

    fn validar(valor: &str) -> Result<TasaValidada, ErrorValidacionTasa> {
        TasaValidada::validar(
            " usd ",
            " ves ",
            valor,
            "2026-10-28",
            " BCV ",
            &catalogo(),
            NaiveDate::from_ymd_opt(2026, 10, 28).unwrap(),
        )
    }

    #[test]
    fn normaliza_y_acepta_tasa_valida_con_ocho_decimales() {
        let tasa = validar("887.12345678").unwrap();

        assert_eq!(tasa.moneda_origen, "USD");
        assert_eq!(tasa.moneda_destino, "VES");
        assert_eq!(tasa.valor, Decimal::from_str("887.12345678").unwrap());
        assert_eq!(tasa.fuente, "BCV");
    }

    #[test]
    fn rechaza_tasa_con_mas_de_ocho_decimales() {
        assert_eq!(
            validar("887.123456789").unwrap_err(),
            ErrorValidacionTasa::DecimalesExcedidos
        );
    }

    #[test]
    fn rechaza_fecha_vacia_y_fuente_vacia_en_el_dominio() {
        let fecha = TasaValidada::validar(
            "USD",
            "VES",
            "1",
            "",
            "BCV",
            &catalogo(),
            NaiveDate::from_ymd_opt(2026, 10, 28).unwrap(),
        );
        assert_eq!(fecha.unwrap_err(), ErrorValidacionTasa::FechaVacia);

        let fuente = TasaValidada::validar(
            "USD",
            "VES",
            "1",
            "2026-10-28",
            "  ",
            &catalogo(),
            NaiveDate::from_ymd_opt(2026, 10, 28).unwrap(),
        );
        assert_eq!(fuente.unwrap_err(), ErrorValidacionTasa::FuenteVacia);
    }
}
