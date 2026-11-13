use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

/// Participante con su parte calculada
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParteCalculada {
    pub username: String,
    pub monto: Decimal,
    pub tiene_redondeo: bool,
}

/// Regla de división de un gasto compartido
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReglaDivision {
    PartesIguales { participantes: Vec<String> },
    Porcentajes { porcentajes: Vec<(String, Decimal)> },
    MontosFijos { montos: Vec<(String, Decimal)> },
}

/// Calcula la división de un gasto según la regla especificada
/// Cumple INV-07: la suma de las partes es exactamente el monto total
/// Cumple la regla de residuo: los céntimos sobrantes se reparten en orden alfabético
pub fn calcular_division(
    monto_total: Decimal,
    regla: &ReglaDivision,
) -> Result<Vec<ParteCalculada>, String> {
    validar_monto_total(monto_total)?;

    match regla {
        ReglaDivision::PartesIguales { participantes } => {
            dividir_partes_iguales(monto_total, participantes)
        }
        ReglaDivision::Porcentajes { porcentajes } => {
            dividir_por_porcentajes(monto_total, porcentajes)
        }
        ReglaDivision::MontosFijos { montos } => dividir_por_montos_fijos(monto_total, montos),
    }
}

fn validar_monto_total(monto_total: Decimal) -> Result<(), String> {
    if monto_total < Decimal::ZERO {
        return Err("El monto total no puede ser negativo".to_string());
    }

    if truncar_a_centavos(monto_total) != monto_total {
        return Err("El monto total debe tener como máximo dos decimales".to_string());
    }

    Ok(())
}

fn truncar_a_centavos(monto: Decimal) -> Decimal {
    (monto * Decimal::new(100, 0)).trunc() / Decimal::new(100, 0)
}

fn validar_participantes(participantes: &[String]) -> Result<(), String> {
    if participantes.is_empty() {
        return Err("Debe haber al menos un participante".to_string());
    }

    let mut nombres = participantes.to_vec();
    nombres.sort();
    if nombres.windows(2).any(|pares| pares[0] == pares[1]) {
        return Err("No puede haber participantes repetidos".to_string());
    }

    Ok(())
}

fn repartir_residuo(
    partes_truncadas: Vec<(String, Decimal)>,
    monto_total: Decimal,
) -> Result<Vec<ParteCalculada>, String> {
    let suma_truncada: Decimal = partes_truncadas.iter().map(|(_, monto)| *monto).sum();
    let residuo_centavos = ((monto_total - suma_truncada) * Decimal::new(100, 0))
        .trunc()
        .to_i64()
        .ok_or_else(|| "El residuo no puede convertirse a céntimos enteros".to_string())?;

    let mut partes_ordenadas = partes_truncadas;
    partes_ordenadas.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(partes_ordenadas
        .into_iter()
        .enumerate()
        .map(|(indice, (username, monto))| {
            let tiene_redondeo = (indice as i64) < residuo_centavos;
            let monto_final = if tiene_redondeo {
                monto + Decimal::new(1, 2)
            } else {
                monto
            };

            ParteCalculada {
                username,
                monto: monto_final,
                tiene_redondeo,
            }
        })
        .collect())
}

fn dividir_partes_iguales(
    monto_total: Decimal,
    participantes: &[String],
) -> Result<Vec<ParteCalculada>, String> {
    validar_participantes(participantes)?;

    let n = Decimal::new(participantes.len() as i64, 0);
    let parte_base = truncar_a_centavos(monto_total / n);
    let partes_truncadas = participantes
        .iter()
        .map(|username| (username.clone(), parte_base))
        .collect();

    repartir_residuo(partes_truncadas, monto_total)
}

fn dividir_por_porcentajes(
    monto_total: Decimal,
    porcentajes: &[(String, Decimal)],
) -> Result<Vec<ParteCalculada>, String> {
    if porcentajes.is_empty() {
        return Err("Debe haber al menos un porcentaje".to_string());
    }

    validar_participantes(
        &porcentajes
            .iter()
            .map(|(username, _)| username.clone())
            .collect::<Vec<_>>(),
    )?;

    if porcentajes.iter().any(|(_, porcentaje)| *porcentaje < Decimal::ZERO) {
        return Err("Los porcentajes no pueden ser negativos".to_string());
    }

    let suma_porcentajes: Decimal = porcentajes.iter().map(|(_, p)| *p).sum();
    if suma_porcentajes != Decimal::new(100, 0) {
        return Err(format!(
            "Los porcentajes suman {}; deben sumar exactamente 100",
            suma_porcentajes
        ));
    }

    let partes_truncadas: Vec<(String, Decimal)> = porcentajes
        .iter()
        .map(|(username, porcentaje)| {
            let monto = truncar_a_centavos(monto_total * porcentaje / Decimal::new(100, 0));
            (username.clone(), monto)
        })
        .collect();

    repartir_residuo(partes_truncadas, monto_total)
}

fn dividir_por_montos_fijos(
    monto_total: Decimal,
    montos: &[(String, Decimal)],
) -> Result<Vec<ParteCalculada>, String> {
    if montos.is_empty() {
        return Err("Debe haber al menos un monto".to_string());
    }

    validar_participantes(
        &montos
            .iter()
            .map(|(username, _)| username.clone())
            .collect::<Vec<_>>(),
    )?;

    if montos.iter().any(|(_, monto)| *monto < Decimal::ZERO) {
        return Err("Los montos no pueden ser negativos".to_string());
    }

    if montos
        .iter()
        .any(|(_, monto)| truncar_a_centavos(*monto) != *monto)
    {
        return Err("Cada monto debe tener como máximo dos decimales".to_string());
    }

    let suma_montos: Decimal = montos.iter().map(|(_, m)| *m).sum();
    if suma_montos != monto_total {
        return Err(format!(
            "Los montos suman {}; deben sumar exactamente {}",
            suma_montos, monto_total
        ));
    }

    let partes = montos
        .iter()
        .map(|(username, monto)| ParteCalculada {
            username: username.clone(),
            monto: *monto,
            tiene_redondeo: false,
        })
        .collect();

    Ok(partes)
}

/// Calcula el balance de un miembro en un grupo
/// Cumple INV-08: la suma de todos los balances es exactamente cero
pub fn calcular_balance(
    username: &str,
    gastos: &[(String, Decimal, Vec<ParteCalculada>)],
) -> Decimal {
    let mut pagado = Decimal::new(0, 2);
    let mut debe = Decimal::new(0, 2);

    for (pagador, monto_pagado, partes) in gastos {
        if pagador == username {
            pagado += *monto_pagado;
        }
        if let Some(parte) = partes.iter().find(|p| p.username == username) {
            debe += parte.monto;
        }
    }

    pagado - debe
}

/// Verifica INV-08: la suma de balances de todos los miembros es cero
pub fn verificar_suma_balances_cero(balances: &[(String, Decimal)]) -> bool {
    let suma: Decimal = balances.iter().map(|(_, b)| *b).sum();
    suma == Decimal::new(0, 2)
}
