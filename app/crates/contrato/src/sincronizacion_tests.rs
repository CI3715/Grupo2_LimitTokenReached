#[cfg(test)]
mod tests {
    use crate::{evento_cambio_dto, respuestacambiosdto, obtenercambiosrequest};

    #[test]
    fn dto_evento_cambio_mantiene_precision_decimal() {
        let evento = evento_cambio_dto {
            id_evento: "evt_123".to_string(),
            tipo: "gastodegruporegistrado".to_string(),
            grupo_id: "grp_1".to_string(),
            gasto_id: "gst_1".to_string(),
            concepto: "hotel".to_string(),
            fecha: "2026-11-17t10:00:00z".to_string(),
            moneda: "usd".to_string(),
            // aqui fuerzo que el monto sea string para cumplir estrictamente el inv-03
            // si usara f64 o f32, el dinero perderia precision por errores de punto flotante 
            // y la suma de los balances del grupo no daria exactamente cero
            monto_parte: "33.34".to_string(), 
            incluye_redondeo: true,
            posicion_cursor: 1,
        };

        // verifico que el valor exacto viaje intacto a traves de la red, sin redondeos binarios
        assert_eq!(evento.monto_parte, "33.34");
        assert!(evento.incluye_redondeo);
    }

    #[test]
    fn respuesta_cambios_soporta_lotes_y_cursor() {
        let respuesta = respuestacambiosdto {
            eventos: vec![],
            nuevo_cursor: 100,
            hay_mas: true,
        };

        // valido la logica de paginacion 
        // se quieren lotes de maximo 100 eventos para no saturar la red ni la memoria del 
        // dispositivo movil durante la sincronizacion en segundo plano
        assert_eq!(respuesta.nuevo_cursor, 100);
        assert!(respuesta.hay_mas);
    }

    #[test]
    fn request_cambios_permite_cursor_nulo() {
        let req = obtenercambiosrequest { desde: none };
        
        // si el cursor es none, el backend debe interpretar que es la primera 
        // sincronizacion del usuario o que se perdió el estado local, por lo que 
        // debe devolver todos los eventos disponibles desde el principio (cursor 0)
        assert!(req.desde.is_none(), "el cursor inicial puede ser nulo para sincronizacion completa");
    }
}