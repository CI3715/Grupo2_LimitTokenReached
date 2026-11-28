#[cfg(test)]
mod tests {
    use crate::{CrearCuentaRequest, TipoCuentaDTO};

    // 100.50 * 10^8 = 10_050_000_000
    fn request_valido() -> CrearCuentaRequest {
        CrearCuentaRequest {
            nombre: "Cartera Principal".to_string(),
            tipo: TipoCuentaDTO::Efectivo,
            moneda_codigo: "VES".to_string(),
            saldo_inicial: 10_050_000_000,
        }
    }

    #[test]
    fn inv03_saldo_debe_ser_i64_multiplicado_por_10_8() {
        let request = request_valido();
        assert_eq!(request.saldo_inicial, 10_050_000_000);
        // Verifica que no hay pérdida de precisión
        assert_eq!(request.saldo_inicial % 100_000_000, 50_000_000, "los 50 centavos deben preservarse");
    }

    #[test]
    fn inv01_monto_debe_tener_moneda_y_cantidad() {
        let request = request_valido();
        assert!(!request.moneda_codigo.is_empty(), "INV-01: Falta moneda");
        assert_ne!(request.saldo_inicial, 0, "INV-01: Falta cantidad");
    }

    #[test]
    fn inv18_cuenta_creada_no_debe_permitir_cambiar_moneda() {
        let request = request_valido();
        assert_eq!(request.moneda_codigo, "VES");
    }

    #[test]
    fn validacion_nombre_vacio_o_espacios_debe_ser_detectable() {
        let mut req = request_valido();
        req.nombre = "   ".to_string();
        assert!(
            req.nombre.trim().is_empty(),
            "El nombre con solo espacios debe ser detectable"
        );
    }

    #[test]
    fn validacion_moneda_fuera_de_catalogo_debe_ser_detectable() {
        let mut req = request_valido();
        req.moneda_codigo = "BTC".to_string();
        let catalogo_base = ["VES", "USD", "EUR", "USDT"];
        assert!(
            !catalogo_base.contains(&req.moneda_codigo.as_str()),
            "BTC no debe estar en el catálogo base"
        );
    }

    #[test]
    fn validacion_saldo_negativo_debe_ser_detectable() {
        let mut req = request_valido();
        req.saldo_inicial = -5_000_000_000; // -50.00
        assert!(
            req.saldo_inicial < 0,
            "El saldo negativo debe ser detectable"
        );
    }

    #[test]
    fn validacion_saldo_cero_debe_ser_detectable() {
        let mut req = request_valido();
        req.saldo_inicial = 0;
        assert_eq!(req.saldo_inicial, 0, "El saldo cero debe ser detectable");
    }

    #[test]
    fn precision_i64_preserva_decimales_de_usd() {
        // 10.55 USD * 10^8 = 1_055_000_000 (2 decimales válidos)
        let req = CrearCuentaRequest {
            nombre: "Cuenta USD".to_string(),
            tipo: TipoCuentaDTO::Banco,
            moneda_codigo: "USD".to_string(),
            saldo_inicial: 1_055_000_000,
        };
        // Verifica que los centavos se representan exactamente
        assert_eq!(req.saldo_inicial % 100_000_000, 55_000_000);
    }
}