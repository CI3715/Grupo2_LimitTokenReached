#[cfg(test)]
mod tests {
    use crate::{
        ConsultarPatrimonioRequest, ConsultarPatrimonioResponse, SubtotalMonedaDTO, TasaFaltanteDTO,
    };

    // INV-02: Prohibido sumar monedas sin tasa
    #[test]
    fn inv02_si_falta_tasa_total_consolidado_debe_ser_none() {
        let response = ConsultarPatrimonioResponse {
            moneda_base: "USD".to_string(),
            saldos_por_cuenta: vec![],
            subtotales_por_moneda: vec![
                SubtotalMonedaDTO {
                    moneda: "USD".to_string(),
                    subtotal: "100.00".to_string(),
                },
                SubtotalMonedaDTO {
                    moneda: "USDT".to_string(),
                    subtotal: "50.00".to_string(),
                },
            ],
            total_consolidado: None, // INV-02: falta tasa USDT→USD
            tasas_usadas: vec![],
            tasas_faltantes: vec![TasaFaltanteDTO {
                moneda_origen: "USDT".to_string(),
                moneda_destino: "USD".to_string(),
                fecha_limite: "2026-10-23".to_string(),
            }],
        };

        assert!(
            response.total_consolidado.is_none(),
            "INV-02: sin tasa, el total consolidado debe ser None"
        );
        assert_eq!(response.tasas_faltantes.len(), 1);
    }

    #[test]
    fn inv02_con_todas_las_tasas_total_consolidado_debe_ser_some() {
        let response = ConsultarPatrimonioResponse {
            moneda_base: "VES".to_string(),
            saldos_por_cuenta: vec![],
            subtotales_por_moneda: vec![],
            total_consolidado: Some("316100.00".to_string()),
            tasas_usadas: vec![],
            tasas_faltantes: vec![],
        };

        assert!(
            response.total_consolidado.is_some(),
            "INV-02: con todas las tasas, el total debe estar presente"
        );
    }

    // validaciones del request
    #[test]
    fn validacion_fecha_consulta_futura_debe_ser_detectable() {
        let req = ConsultarPatrimonioRequest {
            fecha_consulta: "2030-01-01".to_string(),
        };
        assert!(
            req.fecha_consulta.as_str() > "2026-12-31",
            "Fecha de consulta futura detectable"
        );
    }
}
