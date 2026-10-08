use std::future::Future;
use std::time::Instant;

/// Ejecuta la función de petición pasada como argumento y calcula
/// el tiempo de latencia transcurrido en milisegundos.
pub async fn calcular_latencia_ping<F, Fut, T, E>(hacer_peticion: F) -> Result<(T, u128), E>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    // 1. Iniciar el cronómetro antes de llamar a la red
    let inicio = Instant::now();

    // 2. Ejecutar la llamada asíncrona enviada por el adaptador
    let resultado = hacer_peticion().await?;

    // 3. Obtener el tiempo transcurrido en milisegundos
    let latencia_ms = inicio.elapsed().as_millis();

    // 4. Devolver la respuesta intacta junto con los milisegundos de latencia
    Ok((resultado, latencia_ms))
}