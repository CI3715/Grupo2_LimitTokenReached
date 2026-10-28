use crate::puertos::{
    Reloj, RepositorioMonedas, RepositorioPatrimonio, RepositorioPreferencias, RepositorioTasas,
};
use chrono::NaiveDate;
use contrato::{
    error_validacion::ErrorValidacionDTO,
    patrimonio::{
        ConsultarPatrimonioRequest, ConsultarPatrimonioResponse, SubtotalMonedaDTO,
        TasaFaltanteDTO, TasaUsadaDTO,
    },
};
use rust_decimal::Decimal;
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    str::FromStr,
};

pub async fn ejecutar(
    datos: ConsultarPatrimonioRequest,
    repo_patrimonio: &dyn RepositorioPatrimonio,
    repo_tasas: &dyn RepositorioTasas,
    repo_preferencias: &dyn RepositorioPreferencias,
    repo_monedas: &dyn RepositorioMonedas,
    reloj: &dyn Reloj,
) -> Result<ConsultarPatrimonioResponse, ErrorValidacionDTO> {
    let fecha = NaiveDate::parse_from_str(&datos.fecha_consulta, "%Y-%m-%d").map_err(|_| {
        error("fecha_consulta", "Formato inválido. Use AAAA-MM-DD.")
    })?;
    let hoy = reloj.ahora().get(..10)
        .and_then(|fecha| NaiveDate::parse_from_str(fecha, "%Y-%m-%d").ok())
        .ok_or_else(|| error("fecha_consulta", "No se pudo obtener la fecha actual."))?;
    if fecha > hoy {
        return Err(error(
            "fecha_consulta",
            "La fecha de consulta no puede ser futura.",
        ));
    }

    let moneda_base = repo_preferencias
        .obtener_moneda_base()
        .map_err(error_general)?;
    let monedas_catalogo = repo_monedas.obtener_monedas().map_err(error_general)?;
    let decimales_base = monedas_catalogo
        .iter()
        .find(|moneda| moneda.codigo == moneda_base)
        .map(|moneda| moneda.decimales)
        .ok_or_else(|| error_general(format!("La moneda base '{moneda_base}' no existe.")))?;
    let saldos = repo_patrimonio
        .obtener_saldos_a_fecha(fecha)
        .map_err(error_general)?;
    let monedas: BTreeMap<String, Decimal> = {
        let mut totales = BTreeMap::new();
        for saldo in &saldos {
            let monto = Decimal::from_str(&saldo.saldo)
                .map_err(|_| error_general("Se encontró un saldo inválido.".to_string()))?;
            *totales.entry(saldo.moneda.clone()).or_insert(Decimal::ZERO) += monto;
        }
        totales
    };
    let tasas = repo_tasas
        .obtener_tasas_hasta(fecha)
        .map_err(error_general)?;

    let mut subtotales_por_moneda = Vec::with_capacity(monedas.len());
    let mut total = Decimal::ZERO;
    let mut tasas_usadas = Vec::new();
    let mut tasas_faltantes = Vec::new();

    for (moneda, subtotal) in monedas {
        subtotales_por_moneda.push(SubtotalMonedaDTO {
            moneda: moneda.clone(),
            subtotal: subtotal.to_string(),
        });
        if moneda == moneda_base {
            total += subtotal;
            continue;
        }
        if subtotal.is_zero() {
            continue;
        }

        match encontrar_conversion(&moneda, &moneda_base, &tasas)? {
            Some((factor, camino)) => {
                total += subtotal * factor;
                tasas_usadas.extend(camino);
            }
            None => tasas_faltantes.push(TasaFaltanteDTO {
                moneda_origen: moneda,
                moneda_destino: moneda_base.clone(),
                fecha_limite: fecha.to_string(),
            }),
        }
    }

    Ok(ConsultarPatrimonioResponse {
        moneda_base,
        saldos_por_cuenta: saldos,
        subtotales_por_moneda,
        total_consolidado: if tasas_faltantes.is_empty() {
            Some(
                total
                    .round_dp(decimales_base)
                    .normalize()
                    .to_string(),
            )
        } else {
            None
        },
        tasas_usadas,
        tasas_faltantes,
    })
}

fn encontrar_conversion(
    origen: &str,
    destino: &str,
    tasas: &[contrato::tasa::TasaDTO],
) -> Result<Option<(Decimal, Vec<TasaUsadaDTO>)>, ErrorValidacionDTO> {
    let mut pares = HashSet::new();
    let mut aristas = Vec::new();
    for tasa in tasas {
        let valor = Decimal::from_str(&tasa.valor)
            .map_err(|_| error_general(format!("La tasa {} tiene un valor inválido.", tasa.id)))?;
        if valor <= Decimal::ZERO {
            return Err(error_general(format!(
                "La tasa {} debe ser mayor que cero.",
                tasa.id
            )));
        }
        let clave = (tasa.moneda_origen.clone(), tasa.moneda_destino.clone());
        if !pares.insert(clave) {
            continue;
        }
        aristas.push((
            tasa.moneda_origen.clone(),
            tasa.moneda_destino.clone(),
            valor,
            TasaUsadaDTO {
                moneda_origen: tasa.moneda_origen.clone(),
                moneda_destino: tasa.moneda_destino.clone(),
                valor: tasa.valor.clone(),
                fecha: tasa.fecha.clone(),
                fuente: tasa.fuente.clone(),
                es_inversa: false,
            },
        ));
        let inversa = TasaUsadaDTO {
            moneda_origen: tasa.moneda_destino.clone(),
            moneda_destino: tasa.moneda_origen.clone(),
            valor: tasa.valor.clone(),
            fecha: tasa.fecha.clone(),
            fuente: tasa.fuente.clone(),
            es_inversa: true,
        };
        aristas.push((
            tasa.moneda_destino.clone(),
            tasa.moneda_origen.clone(),
            Decimal::ONE / valor,
            inversa,
        ));
    }
    aristas.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));

    let mut visitados = HashSet::from([origen.to_string()]);
    let mut pendientes = VecDeque::from([(
        origen.to_string(),
        Decimal::ONE,
        Vec::<TasaUsadaDTO>::new(),
    )]);
    while let Some((moneda, factor, camino)) = pendientes.pop_front() {
        for (_, hasta, valor, detalle) in aristas.iter().filter(|edge| edge.0 == moneda) {
            if !visitados.insert(hasta.clone()) {
                continue;
            }
            let mut nuevo_camino = camino.clone();
            nuevo_camino.push(detalle.clone());
            let nuevo_factor = factor * valor;
            if hasta == destino {
                return Ok(Some((nuevo_factor, nuevo_camino)));
            }
            pendientes.push_back((hasta.clone(), nuevo_factor, nuevo_camino));
        }
    }
    Ok(None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use contrato::tasa::TasaDTO;

    fn tasa(origen: &str, destino: &str, valor: &str) -> TasaDTO {
        TasaDTO {
            id: "1".to_string(),
            moneda_origen: origen.to_string(),
            moneda_destino: destino.to_string(),
            valor: valor.to_string(),
            fecha: "2026-10-27".to_string(),
            fuente: "test".to_string(),
            eliminada: false,
        }
    }

    #[test]
    fn convierte_por_un_camino_de_tasas_y_retorna_sus_detalles() {
        let tasas = vec![
            tasa("EUR", "VES", "1000"),
            tasa("VES", "USD", "0.001"),
        ];
        let (factor, camino) = encontrar_conversion("EUR", "USD", &tasas)
            .unwrap()
            .unwrap();
        assert_eq!(factor, Decimal::ONE);
        assert_eq!(camino.len(), 2);
    }

    #[test]
    fn usa_la_inversa_si_no_hay_tasa_en_la_direccion_requerida() {
        let tasas = vec![tasa("USD", "VES", "1000")];
        let (factor, camino) = encontrar_conversion("VES", "USD", &tasas)
            .unwrap()
            .unwrap();
        assert_eq!(factor, Decimal::new(1, 3));
        assert!(camino[0].es_inversa);
    }

    #[test]
    fn no_inventa_conversion_si_no_hay_camino() {
        assert!(
            encontrar_conversion("EUR", "USD", &[tasa("VES", "USD", "1000")])
                .unwrap()
                .is_none()
        );
    }
}
