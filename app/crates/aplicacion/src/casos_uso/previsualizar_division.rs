use std::collections::HashSet;
use std::str::FromStr;

use contrato::{ParteGastoDTO, RegistrarGastoRequestDTO};
use division::{ReglaDivision, calcular_division};
use rust_decimal::Decimal;

fn decimal(valor: &str, campo: &str) -> Result<Decimal, String> {
    Decimal::from_str(valor)
        .map_err(|_| format!("El valor de {campo} debe ser un decimal válido."))
}

fn validar_valores_extra(
    participantes: &[String],
    valores: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let participantes: HashSet<&str> = participantes.iter().map(String::as_str).collect();
    if let Some(usuario) = valores
        .keys()
        .map(String::as_str)
        .find(|usuario| !participantes.contains(usuario))
    {
        return Err(format!("El usuario {usuario} no es participante del gasto."));
    }

    Ok(())
}

fn construir_regla(datos: &RegistrarGastoRequestDTO) -> Result<ReglaDivision, String> {
    validar_valores_extra(&datos.participantes, &datos.valores)?;

    match datos.regla.as_str() {
        "iguales" => {
            if !datos.valores.is_empty() {
                return Err(
                    "La división en partes iguales no debe recibir valores adicionales."
                        .to_string(),
                );
            }
            Ok(ReglaDivision::PartesIguales {
                participantes: datos.participantes.clone(),
            })
        }
        "porcentajes" | "fijos" => {
            let valores = datos
                .participantes
                .iter()
                .map(|usuario| {
                    let valor = datos
                        .valores
                        .get(usuario)
                        .ok_or_else(|| format!("Falta el valor del participante {usuario}."))?;
                    Ok((usuario.clone(), decimal(valor, "la división")?))
                })
                .collect::<Result<Vec<_>, String>>()?;

            if datos.regla == "porcentajes" {
                Ok(ReglaDivision::Porcentajes { porcentajes: valores })
            } else {
                Ok(ReglaDivision::MontosFijos { montos: valores })
            }
        }
        _ => Err("La regla de división no es válida.".to_string()),
    }
}

pub fn ejecutar(datos: RegistrarGastoRequestDTO) -> Result<Vec<ParteGastoDTO>, String> {
    let monto_total = decimal(&datos.monto, "el monto")?;
    let regla = construir_regla(&datos)?;
    let partes = calcular_division(monto_total, &regla)?;

    Ok(partes
        .into_iter()
        .map(|parte| ParteGastoDTO {
            usuario: parte.username,
            monto: parte.monto.to_string(),
            residuo: parte.tiene_redondeo,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn datos(regla: &str) -> RegistrarGastoRequestDTO {
        RegistrarGastoRequestDTO {
            concepto: "Cena".to_string(),
            monto: "0.05".to_string(),
            fecha: "2026-11-12".to_string(),
            pagador: "beto".to_string(),
            regla: regla.to_string(),
            participantes: vec!["beto".to_string(), "ana".to_string()],
            valores: std::collections::HashMap::new(),
        }
    }

    fn valores(datos: &mut RegistrarGastoRequestDTO, valores: &[(&str, &str)]) {
        datos.valores = valores
            .iter()
            .map(|(usuario, valor)| (usuario.to_string(), valor.to_string()))
            .collect();
    }

    #[test]
    fn vista_previa_iguales_devuelve_residuo_y_orden_alfabetico() {
        let resultado = ejecutar(datos("iguales")).unwrap();

        assert_eq!(resultado[0].usuario, "ana");
        assert_eq!(resultado[0].monto, "0.03");
        assert!(resultado[0].residuo);
        assert_eq!(resultado[1].usuario, "beto");
        assert_eq!(resultado[1].monto, "0.02");
        assert!(!resultado[1].residuo);
    }

    #[test]
    fn vista_previa_rechaza_valor_de_participante_ausente() {
        let mut solicitud = datos("iguales");
        solicitud.valores.insert("carla".to_string(), "50".to_string());

        assert!(ejecutar(solicitud).is_err());
    }

    #[test]
    fn vista_previa_porcentajes_convierte_valores_y_conserva_el_total() {
        let mut solicitud = datos("porcentajes");
        valores(&mut solicitud, &[("ana", "50"), ("beto", "50")]);

        let resultado = ejecutar(solicitud).unwrap();

        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].monto, "0.03");
        assert_eq!(resultado[1].monto, "0.02");
        assert_eq!(
            resultado.iter().map(|parte| parte.monto.parse::<Decimal>().unwrap()).sum::<Decimal>(),
            Decimal::new(5, 2)
        );
    }

    #[test]
    fn vista_previa_montos_fijos_devuelve_los_montos_indicados() {
        let mut solicitud = datos("fijos");
        valores(&mut solicitud, &[("ana", "0.03"), ("beto", "0.02")]);

        let resultado = ejecutar(solicitud).unwrap();
        let ana = resultado.iter().find(|parte| parte.usuario == "ana").unwrap();
        let beto = resultado.iter().find(|parte| parte.usuario == "beto").unwrap();

        assert_eq!(ana.monto, "0.03");
        assert_eq!(beto.monto, "0.02");
        assert!(!ana.residuo);
        assert!(!beto.residuo);
    }

    #[test]
    fn vista_previa_rechaza_monto_invalido() {
        let mut solicitud = datos("iguales");
        solicitud.monto = "no-es-un-decimal".to_string();

        assert!(ejecutar(solicitud).is_err());
    }

    #[test]
    fn vista_previa_rechaza_regla_desconocida() {
        assert!(ejecutar(datos("desconocida")).is_err());
    }

    #[test]
    fn vista_previa_rechaza_valor_faltante() {
        let mut solicitud = datos("porcentajes");
        valores(&mut solicitud, &[("ana", "50")]);

        assert!(ejecutar(solicitud).is_err());
    }
}
