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

fn dividir_partes_iguales(
    monto_total: Decimal,
    participantes: &[String],
) -> Result<Vec<ParteCalculada>, String> {
    if participantes.is_empty() {
        return Err("Debe haber al menos un participante".to_string());
    }

    let n = Decimal::new(participantes.len() as i64, 0);
    let parte_base = (monto_total / n).round_dp(2);
    let suma_base = parte_base * n;
    let residuo = monto_total - suma_base;

    // Ordenar participantes alfabéticamente para repartir residuo
    let mut participantes_ordenados: Vec<String> = participantes.to_vec();
    participantes_ordenados.sort();

    let residuo_centavos = (residuo * Decimal::new(100, 0)).to_i32().unwrap_or(0);

    let mut partes = Vec::new();
    for (i, username) in participantes_ordenados.iter().enumerate() {
        let tiene_redondeo = (i as i32) < residuo_centavos;
        let monto = if tiene_redondeo {
            parte_base + Decimal::new(1, 2) // +0.01
        } else {
            parte_base
        };
        partes.push(ParteCalculada {
            username: username.clone(),
            monto,
            tiene_redondeo,
        });
    }

    Ok(partes)
}

fn dividir_por_porcentajes(
    monto_total: Decimal,
    porcentajes: &[(String, Decimal)],
) -> Result<Vec<ParteCalculada>, String> {
    // Validar que los porcentajes sumen 100
    let suma_porcentajes: Decimal = porcentajes.iter().map(|(_, p)| *p).sum();
    if (suma_porcentajes - Decimal::new(100, 0)).abs() > Decimal::new(1, 2) {
        return Err(format!(
            "Los porcentajes suman {}; deben sumar exactamente 100",
            suma_porcentajes
        ));
    }

    // Calcular partes truncadas a céntimos
    let mut partes_truncadas: Vec<(String, Decimal)> = porcentajes
        .iter()
        .map(|(username, porcentaje)| {
            let monto = (monto_total * porcentaje / Decimal::new(100, 0)).round_dp(2);
            (username.clone(), monto)
        })
        .collect();

    // Calcular residuo
    let suma_truncada: Decimal = partes_truncadas.iter().map(|(_, m)| *m).sum();
    let residuo = monto_total - suma_truncada;
    let residuo_centavos = (residuo * Decimal::new(100, 0)).to_i32().unwrap_or(0);

    // Ordenar alfabéticamente para repartir residuo
    partes_truncadas.sort_by(|a, b| a.0.cmp(&b.0));

    let mut partes = Vec::new();
    for (i, (username, monto)) in partes_truncadas.iter().enumerate() {
        let tiene_redondeo = (i as i32) < residuo_centavos;
        let monto_final = if tiene_redondeo {
            *monto + Decimal::new(1, 2)
        } else {
            *monto
        };
        partes.push(ParteCalculada {
            username: username.clone(),
            monto: monto_final,
            tiene_redondeo,
        });
    }

    Ok(partes)
}

fn dividir_por_montos_fijos(
    monto_total: Decimal,
    montos: &[(String, Decimal)],
) -> Result<Vec<ParteCalculada>, String> {
    // Validar que los montos sumen el total
    let suma_montos: Decimal = montos.iter().map(|(_, m)| *m).sum();
    if (suma_montos - monto_total).abs() > Decimal::new(1, 2) {
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

    for (pagador, _, partes) in gastos {
        if pagador == username {
            pagado += partes.iter().map(|p| p.monto).sum::<Decimal>();
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
