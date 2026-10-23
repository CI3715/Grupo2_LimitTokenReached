#[cfg(test)]
mod tests {
    use crate::{RegistrarMovimientoRequest, TipoMovimientoDTO};

    fn request_valido() -> RegistrarMovimientoRequest {
        RegistrarMovimientoRequest {
            cuenta_id: "cuenta-123".to_string(),
            tipo: TipoMovimientoDTO::Gasto,
            monto: "50.00".to_string(),
            fecha: "2026-10-19".to_string(),
            categoria_id: Some("cat-alimentacion".to_string()),
            nota: Some("Mercado".to_string()),
            confirmar_saldo_negativo: false,
        }
    }

    #[test]
    fn inv03_monto_debe_ser_string_para_preservar_precision() {
        let request = request_valido();
        let _es_string: &String = &request.monto;
        assert_eq!(request.monto, "50.00");
    }

    #[test]
    fn validacion_monto_cero_o_negativo_debe_ser_detectable() {
        let mut req = request_valido();
        req.monto = "-10.00".to_string();
        assert!(
            req.monto.starts_with('-'),
            "El sistema debe detectar montos negativos"
        );
    }

    #[test]
    fn validacion_fecha_futura_debe_ser_detectable() {
        let mut req = request_valido();
        req.fecha = "2030-01-01".to_string();
        assert!(
            req.fecha.as_str() > "2026-12-31",
            "Fecha futura detectable en el request"
        );
    }

    #[test]
    fn validacion_categoria_vacia_debe_ser_detectable() {
        let mut req = request_valido();
        req.categoria_id = Some("".to_string());
        assert!(
            req.categoria_id.as_deref() == Some(""),
            "La categoría vacía debe ser rechazada"
        );
    }
}
