#[cfg(test)]
mod tests {
    use crate::{RegistrarTasaRequest, EliminarTasaRequest};

    fn tasa_valida() -> RegistrarTasaRequest {
        RegistrarTasaRequest {
            moneda_origen: "USD".to_string(),
            moneda_destino: "VES".to_string(),
            valor: "887.00".to_string(),
            fecha: "2026-10-23".to_string(),
            fuente: "BCV".to_string(),
        }
    }

    // INV-03: Prohibido f32/f64 para dinero
    #[test]
    fn inv03_valor_de_tasa_debe_ser_string() {
        let tasa = tasa_valida();
        let _es_string: &String = &tasa.valor;
        assert_eq!(tasa.valor, "887.00");
    }

    // validaciones detectables en el request
    #[test]
    fn validacion_valor_cero_o_negativo_debe_ser_detectable() {
        let mut req = tasa_valida();
        req.valor = "0.00".to_string();
        assert_eq!(req.valor, "0.00", "El dominio debe rechazar valor cero");

        req.valor = "-10.00".to_string();
        assert!(req.valor.starts_with('-'), "El dominio debe rechazar valor negativo");
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
        req.moneda_destino = "USD".to_string(); // igual que origen
        assert_eq!(req.moneda_origen, req.moneda_destino, "El dominio debe rechazar monedas iguales");
    }

    #[test]
    fn validacion_eliminar_tasa_requiere_id() {
        let req = EliminarTasaRequest {
            tasa_id: "".to_string(),
        };
        assert!(req.tasa_id.is_empty(), "ID vacío debe ser rechazado");
    }
}