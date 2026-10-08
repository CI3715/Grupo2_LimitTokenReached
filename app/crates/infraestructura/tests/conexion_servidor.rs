#[tokio::test]
async fn verificar_conexion_servidor() {
// 1. Obtenemos la URL base del servidor (Backend FastAPI)
let url_base = option_env!("SERVER_HOST_IP").unwrap_or("http://localhost:8000");

// Limpiamos posibles barras diagonales sobrantes al final para evitar el error de doble barra '//ping'
let url_base_limpia = url_base.trim_end_matches('/');

// 2. Construimos el endpoint pasando la URL como Query Parameter (?url=...)
// FastAPI leerá el parámetro "url" automáticamente desde aquí
let url_endpoint = format!("{}/ping?url={}", url_base_limpia, url_base_limpia);

println!("Probando conexión hacia: {}", url_endpoint);

// 3. Realizamos la petición GET
let respuesta = reqwest::get(&url_endpoint)
.await
.expect("No se pudo conectar con el servidor backend");

// 4. Imprimimos el código de estado HTTP recibido
println!("STATUS HTTP: {}", respuesta.status());

// 5. Imprime la respuesta recibida
let texto = respuesta
.text()
.await
.expect("No se pudo leer el cuerpo de la respuesta");

println!("CUERPO RECIBIDO (TEXTO PLANO):\n{}", texto);
}