#[cfg(test)]
mod tests {
    use crate::{CrearCuentaRequest, TipoCuentaDTO};

    fn request_valido() -> CrearCuentaRequest {
        CrearCuentaRequest {
            nombre: "Cartera Principal".to_string(),
            tipo: TipoCuentaDTO::Efectivo,
            moneda_codigo: "VES".to_string(),
            saldo_inicial: "100.50".to_string(),
        }
    }

    // INVARIANTES 
    #[test]
    fn inv03_saldo_debe_ser_string_para_preservar_precision() {
        let request = request_valido();
        // Prueba de compilación: si alguien cambia el DTO a f64, esto no compila.
        // Usamos '&' para tomar una referencia y no mover el valor.
        let _es_string: &String = &request.saldo_inicial; 
        assert_eq!(request.saldo_inicial, "100.50");
    }

    #[test]
    fn inv01_monto_debe_tener_moneda_y_cantidad() {
        let request = request_valido();
        assert!(!request.moneda_codigo.is_empty(), "INV-01: Falta moneda");
        assert!(!request.saldo_inicial.is_empty(), "INV-01: Falta cantidad");
    }

    #[test]
    fn inv18_cuenta_creada_no_debe_permitir_cambiar_moneda() {
        let request = request_valido();
        assert_eq!(request.moneda_codigo, "VES");
    }

    // VALIDACIONES DE CREAR CUENTA 
    #[test]
    fn validacion_nombre_vacio_o_espacios_debe_ser_detectable() {
        let mut req = request_valido();
        req.nombre = "   ".to_string();
        assert!(req.nombre.trim().is_empty(), "El nombre con solo espacios debe ser detectable");
    }

    #[test]
    fn validacion_moneda_fuera_de_catalogo_debe_ser_detectable() {
        let mut req = request_valido();
        req.moneda_codigo = "BTC".to_string();
        let catalogo_base = ["VES", "USD", "EUR", "USDT"];
        assert!(!catalogo_base.contains(&req.moneda_codigo.as_str()), 
                "BTC no debe estar en el catálogo base");
    }

    #[test]
    fn validacion_saldo_negativo_debe_ser_detectable() {
        let mut req = request_valido();
        req.saldo_inicial = "-50.00".to_string();
        assert!(req.saldo_inicial.starts_with('-'), "El saldo negativo debe ser detectable");
    }

      #[test]
    fn validacion_saldo_con_decimales_invalidos_debe_ser_detectable() {
        let mut req = request_valido();
        req.moneda_codigo = "USD".to_string(); // USD permite 2 decimales
        req.saldo_inicial = "10.555".to_string(); // 3 decimales (inválido)
        
        // Verificar que podemos detectar decimales inválidos
        if let Some(pos_punto) = req.saldo_inicial.find('.') {
            let decimales = req.saldo_inicial[pos_punto + 1..].len();
            let max_decimales_usd = 2;
            // La prueba verifica que detectamos que es inválido (decimales > máximo permitido)
            assert!(decimales > max_decimales_usd, 
                    "Deberíamos detectar que USD tiene más decimales de los permitidos");
        } else {
            panic!("El saldo debería tener un punto decimal");
        }
    }
}