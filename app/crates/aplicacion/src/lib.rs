use std::time::Instant;

// La función en aplicación recibe el resultado que proviene de la infraestructura
pub async fn calcular_latencia_ping<F, Fut>(hacer_peticion: F) -> Result<String, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(String, String, String, String), String>>,
{
    let inicio = Instant::now();
    let (estado, mensaje, version, timestamp) = hacer_peticion().await?;
    let latencia = inicio.elapsed().as_millis();

    Ok(format!(
        "Estado: {}\nMensaje: {}\nVersión: {}\nTimestamp: {}\nLatencia: {} ms",
        estado, mensaje, version, timestamp, latencia
    ))
}