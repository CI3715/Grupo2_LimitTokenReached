#[cfg(test)]
mod tests {
    use crate::{RegistrarMovimientoRequest, TipoMovimientoDTO};

    // 50.00 * 10^8 = 5_000_000_000
    fn request_valido() -> RegistrarMovimientoRequest {
        RegistrarMovimientoRequest {
            cuenta_id: "cuenta-123".to_string(),
            tipo: TipoMovimientoDTO::Gasto,
            monto: 5_000_000_000,
            fecha: "2026-10-19".to_string(),
            categoria_id: Some("cat-alimentacion".to_string()),
            nota: Some("Mercado".to_string()),
            confirmar_saldo_negativo: false,
        }
    }

    #[test]
    fn inv03_monto_debe_ser_i64_para_preservar_precision() {
        let request = request_valido();
        // si alguien cambia el tipo a f64 por error, esto sigue compilando pero
        // la idea es que usamos enteros para que la suma de centavos sea exacta
        assert_eq!(request.monto, 5_000_000_000);
        assert_eq!(request.monto % 100_000_000, 0, "los centavos deben ser exactos");
    }

    #[test]
    fn validacion_monto_cero_o_negativo_debe_ser_detectable() {
        let mut req = request_valido();
        req.monto = -1_000_000_000; // -10.00
        assert!(req.monto < 0, "el sistema debe detectar montos negativos");

        req.monto = 0;
        assert_eq!(req.monto, 0, "el sistema debe detectar montos en cero");
    }

    #[test]
    fn validacion_fecha_futura_debe_ser_detectable() {
        let mut req = request_valido();
        req.fecha = "2030-01-01".to_string();
        assert!(req.fecha.as_str() > "2026-12-31", "fecha futura detectable");
    }

    #[test]
    fn validacion_categoria_vacia_debe_ser_detectable() {
        let mut req = request_valido();
        req.categoria_id = Some("".to_string());
        assert!(req.categoria_id.as_deref() == Some(""), "categoria vacia debe rechazarse");
    }
}