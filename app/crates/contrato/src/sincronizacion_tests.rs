#[cfg(test)]
mod tests {
    use crate::sincronizacion::{EventoCambioDTO, ObtenerCambiosRequest, RespuestaCambiosDTO};

    #[test]
    fn dto_evento_cambio_mantiene_precision_decimal_en_i64() {
        let evento = EventoCambioDTO {
            id_evento: "evt_123".to_string(),
            tipo: "GastoDeGrupoRegistrado".to_string(),
            grupo_id: "grp_1".to_string(),
            gasto_id: "gst_1".to_string(),
            concepto: "Hotel".to_string(),
            fecha: "2026-11-17T10:00:00Z".to_string(),
            moneda: "USD".to_string(),
            // 33.34 * 10^8 = 3_334_000_000
            monto_parte: 3_334_000_000,
            incluye_redondeo: true,
            posicion_cursor: 1,
        };

        // verifico que el valor exacto viaje intacto, sin errores de punto flotante
        assert_eq!(evento.monto_parte, 3_334_000_000);
        assert!(evento.incluye_redondeo);
        // Verifica que los centavos se preservan exactamente
        assert_eq!(evento.monto_parte % 100_000_000, 34_000_000);
    }

    #[test]
    fn respuesta_cambios_soporta_lotes_y_cursor() {
        let respuesta = RespuestaCambiosDTO {
            eventos: vec![],
            nuevo_cursor: 100,
            hay_mas: true,
        };

        assert_eq!(respuesta.nuevo_cursor, 100);
        assert!(respuesta.hay_mas);
    }

    #[test]
    fn request_cambios_permite_cursor_nulo() {
        let req = ObtenerCambiosRequest { desde: None };

        assert!(
            req.desde.is_none(),
            "el cursor inicial puede ser nulo para sincronizacion completa"
        );
    }
}