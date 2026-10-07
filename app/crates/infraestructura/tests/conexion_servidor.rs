#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn probar_conexion_servidor_fastapi() {
        let cliente = ClienteSocialHttp::new("http://localhost:8000");
        let respuesta = cliente.hacer_ping().await.expect("Falló la conexión con el servidor");

        assert_eq!(respuesta.estado, "ok");
    }
}