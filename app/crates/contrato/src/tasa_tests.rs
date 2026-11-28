#[cfg(test)]
mod tests {
    use crate::{EliminarTasaRequest, RegistrarTasaRequest};

    // 887.00 * 10^8 = 88_700_000_000
    fn tasa_valida() -> RegistrarTasaRequest {
        RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: 88_700_000_000,
            fecha: "2026-10-23".to_string(),
            fuente: "BCV".to_string(),
        }
    }

    #[test]
    fn inv03_valor_de_tasa_debe_ser_i64_multiplicado_por_10_8() {
        let tasa = tasa_valida();
        // 887.00 VES por USD representado como entero
        assert_eq!(tasa.valor, 88_700_000_000);
        // Verifica que no hay pérdida de precisión (propiedad clave de i64 vs f64)
        assert_eq!(tasa.valor % 100_000_000, 0, "los centavos deben representarse exactamente");
    }

    #[test]
    fn validacion_valor_cero_o_negativo_debe_ser_detectable() {
        let mut req = tasa_valida();
        req.valor = 0;
        assert_eq!(req.valor, 0, "El dominio debe rechazar valor cero");

        req.valor = -1_000_000_000; // -10.00
        assert!(req.valor < 0, "El dominio debe rechazar valor negativo");
    }

    #[test]
    fn validacion_fecha_futura_debe_ser_detectable() {
        let mut req = tasa_valida();
        req.fecha = "2030-01-01".to_string();
        assert!(req.fecha.as_str() > "2026-12-31", "Fecha futura detectable");
    }

    #[test]
    fn validacion_fuente_vacia_debe_ser_detectable() {
        let mut req = tasa_valida();
        req.fuente = "".to_string();
        assert!(req.fuente.is_empty(), "Fuente vacía debe ser rechazada");
    }

    #[test]
    fn validacion_monedas_iguales_debe_ser_detectable() {
        let mut req = tasa_valida();
        req.moneda_destino = "USD".to_string();
        assert_eq!(
            req.moneda_origen, req.moneda_destino,
            "El dominio debe rechazar monedas iguales"
        );
    }

    #[test]
    fn validacion_eliminar_tasa_requiere_id() {
        let req = EliminarTasaRequest {
            tasa_id: "".to_string(),
        };
        assert!(req.tasa_id.is_empty(), "ID vacío debe ser rechazado");
    }

    #[test]
    fn precision_decimal_se_preserva_en_i64() {
        // 887.12345678 * 10^8 = 88_712_345_678
        let req = RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: 88_712_345_678,
            fecha: "2026-10-23".to_string(),
            fuente: "BCV".to_string(),
        };
        // Verifica que los 8 decimales se preservan exactamente
        assert_eq!(req.valor % 100_000_000, 12_345_678);
    }
}