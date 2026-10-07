use infraestructura::ClienteSocialHttp;

#[tokio::test]
async fn probar_conexion_servidor_fastapi_real() {
    // 1. Lee la URL del servidor desde el archivo .env o usa fallback local
    let url_servidor = option_env!("SERVER_URL");
    let cliente = ClienteSocialHttp::new(url_servidor);

    // 2. Ejecuta la función REAL 'hacer_ping()' que deserializa el JSON
    let respuesta = cliente
        .hacer_ping()
        .await
        .expect("El servidor debe responder con el contrato JSON de la Entrega 1");

    // 3. Evalúa las propiedades reales del contrato exigido
    assert_eq!(respuesta.estado, "ok");
    assert_eq!(respuesta.mensaje, "Servidor Cuentas Claras activo");
    assert_eq!(respuesta.version, "0.1.0");
}