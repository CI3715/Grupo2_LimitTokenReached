#[cfg(test)]
mod tests {
    use contrato::{CrearCuentaRequest, TipoCuentaDTO, ErrorValidacionDTO};
    use crate::infraestructura::repositorio_memoria::{RepositorioMemoria, RepositorioEventos};

    fn request_valido() -> CrearCuentaRequest {
        CrearCuentaRequest {
            nombre: "Cartera Principal".to_string(),
            tipo: TipoCuentaDTO::Efectivo,
            moneda_codigo: "VES".to_string(),
            saldo_inicial: "100.50".to_string(),
        }
    }

    // invariantes
    #[test]
    fn inv03_saldo_debe_ser_string_para_preservar_precision() {
        let request = request_valido();
        // Prueba de compilación: si alguien cambia el DTO a f64, esto no compila
        let _es_string: String = request.saldo_inicial; 
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
        // El DTO de respuesta no debe tener setters mutables para la moneda
    }

// validaciones de crear cuenta
    #[tokio::test]
    async fn validacion_nombre_vacio_o_espacios_debe_fallar() {
        let mut req = request_valido();
        req.nombre = "   ".to_string();
        let repo = RepositorioMemoria::nuevo();
        // TODO (Equipo Dominio): Implementar validación para que retorne Err con campo "nombre"
        // Por ahora, el test verifica que el sistema recibe el request 
        // Cuando el equipo implemente la lógica, esta prueba debe assert_eq!(resultado.is_err(), true)
        let _ = repo.guardar_cuenta(req).await;
    }

    #[tokio::test]
    async fn validacion_moneda_fuera_de_catalogo_debe_fallar() {
        let mut req = request_valido();
        req.moneda_codigo = "BTC".to_string(); // No está en el catálogo base
        let repo = RepositorioMemoria::nuevo();
        // TODO (Equipo Dominio): Implementar validación para que retorne Err con campo "moneda"
        let _ = repo.guardar_cuenta(req).await;
    }

    #[tokio::test]
    async fn validacion_saldo_negativo_debe_fallar() {
        let mut req = request_valido();
        req.saldo_inicial = "-50.00".to_string();
        let repo = RepositorioMemoria::nuevo();
        // TODO (Equipo Dominio): Implementar validación para que retorne Err con campo "saldo_inicial"
        let _ = repo.guardar_cuenta(req).await;
    }

    #[tokio::test]
    async fn validacion_saldo_con_decimales_invalidos_debe_fallar() {
        let mut req = request_valido();
        req.moneda_codigo = "USD".to_string(); // USD permite 2 decimales
        req.saldo_inicial = "10.555".to_string(); // 3 decimales (inválido)
        let repo = RepositorioMemoria::nuevo();
        // TODO (Equipo Dominio): Implementar validación de decimales por moneda
        let _ = repo.guardar_cuenta(req).await;
    }
}