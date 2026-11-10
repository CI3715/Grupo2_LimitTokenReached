//! Transporte HTTP del adaptador real contra un servidor local controlado.
//! No sustituye las pruebas de autorización del servidor FastAPI.
use infraestructura::autenticacion_http::{
    ClienteAutenticacionHttp, InicioSesionHttp, RegistroUsuarioHttp,
};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const PERFIL: &str = r#"{"nombre_usuario":"ana","nombre_visible":"Ana","correo":"ana@example.com","fecha_alta":"2026-11-10T12:00:00Z"}"#;

fn servidor(codigo: u16, cuerpo: &str) -> (String, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let cuerpo = cuerpo.to_owned();
    let hilo = thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let inicio = std::time::Instant::now();
        let (mut socket, _) = loop {
            match listener.accept() {
                Ok(conexion) => break conexion,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(inicio.elapsed() < Duration::from_secs(10), "No llegó HTTP");
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("No se pudo aceptar HTTP: {error}"),
            }
        };
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        loop {
            let mut bloque = [0; 4096];
            let leidos = socket.read(&mut bloque).unwrap();
            assert!(leidos > 0, "Petición incompleta");
            bytes.extend_from_slice(&bloque[..leidos]);
            let texto = String::from_utf8_lossy(&bytes);
            if let Some((cabeceras, contenido)) = texto.split_once("\r\n\r\n") {
                let longitud = cabeceras
                    .lines()
                    .find_map(|linea| {
                        let (nombre, valor) = linea.split_once(':')?;
                        nombre
                            .eq_ignore_ascii_case("content-length")
                            .then(|| valor.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if contenido.len() >= longitud {
                    break;
                }
            }
        }
        write!(socket,
            "HTTP/1.1 {codigo} QA\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
            cuerpo.len()
        ).unwrap();
        String::from_utf8(bytes).unwrap()
    });
    (url, hilo)
}

fn cuerpo(peticion: &str) -> Value {
    serde_json::from_str(peticion.split_once("\r\n\r\n").unwrap().1).unwrap()
}

#[tokio::test]
async fn registro_envia_solo_identidad_al_endpoint_correcto() {
    let (url, hilo) = servidor(201, PERFIL);
    let cliente = ClienteAutenticacionHttp::new(format!("{url}/"));
    let datos = RegistroUsuarioHttp {
        nombre_usuario: "ana".into(),
        nombre_visible: "Ana".into(),
        correo: "ana@example.com".into(),
        contrasena: "Clave exclusiva QA".into(),
    };
    let resultado = cliente.registrar_usuario(&datos).await;
    assert!(resultado.is_ok());
    assert_eq!(resultado.ok().unwrap().nombre_usuario, "ana");
    let peticion = hilo.join().unwrap();
    assert!(peticion.starts_with("POST /usuarios HTTP/1.1\r\n"));
    assert!(!peticion.to_lowercase().contains("authorization:"));
    assert_eq!(
        cuerpo(&peticion),
        json!({
            "nombre_usuario":"ana", "nombre_visible":"Ana", "correo":"ana@example.com",
            "contrasena":"Clave exclusiva QA"
        })
    );
}

#[tokio::test]
async fn login_envia_credenciales_y_lee_token_y_vencimiento() {
    let (url, hilo) = servidor(
        200,
        r#"{"token":"token-qa","vence_en":"2026-11-10T13:00:00Z"}"#,
    );
    let resultado = ClienteAutenticacionHttp::new(url)
        .iniciar_sesion(&InicioSesionHttp {
            correo: "ana@example.com".into(),
            contrasena: "Clave exclusiva QA".into(),
        })
        .await;
    assert!(resultado.is_ok());
    let sesion = resultado.ok().unwrap();
    assert_eq!(sesion.token, "token-qa");
    assert_eq!(sesion.vence_en, "2026-11-10T13:00:00Z");
    let peticion = hilo.join().unwrap();
    assert!(peticion.starts_with("POST /sesiones HTTP/1.1\r\n"));
    assert_eq!(
        cuerpo(&peticion),
        json!({
            "correo":"ana@example.com", "contrasena":"Clave exclusiva QA"
        })
    );
}

#[tokio::test]
async fn perfil_envia_bearer_en_cabecera_y_nunca_en_url() {
    let (url, hilo) = servidor(200, PERFIL);
    let resultado = ClienteAutenticacionHttp::new(url)
        .obtener_perfil("token-qa")
        .await;
    assert!(resultado.is_ok());
    let peticion = hilo.join().unwrap();
    assert!(peticion.starts_with("GET /usuarios/yo HTTP/1.1\r\n"));
    assert!(
        peticion
            .to_lowercase()
            .contains("authorization: bearer token-qa\r\n")
    );
    assert!(!peticion.lines().next().unwrap().contains("token-qa"));
}

#[tokio::test]
async fn conserva_400_401_403_404_409_y_error_de_campo() {
    for codigo in [400, 401, 403, 404, 409] {
        let (url, hilo) = servidor(
            codigo,
            r#"{"campo":"sesion","mensaje":"Acceso rechazado."}"#,
        );
        let resultado = ClienteAutenticacionHttp::new(url)
            .obtener_perfil("token-qa")
            .await;
        assert!(resultado.is_err());
        let error = resultado.err().unwrap();
        assert_eq!(error.status().unwrap().as_u16(), codigo);
        assert_eq!(
            serde_json::from_str::<Value>(&error.mensaje_ui()).unwrap(),
            json!({"campo":"sesion", "mensaje":"Acceso rechazado."})
        );
        hilo.join().unwrap();
    }
}

#[tokio::test]
async fn error_inesperado_no_expone_cuerpo_interno_del_servidor() {
    let (url, hilo) = servidor(500, "<html>error-interno-con-secreto-qa</html>");
    let resultado = ClienteAutenticacionHttp::new(url)
        .obtener_perfil("token-qa")
        .await;
    assert!(resultado.is_err());
    let error = resultado.err().unwrap();
    assert_eq!(error.status().unwrap().as_u16(), 500);
    assert!(!error.mensaje_ui().contains("secreto-qa"));
    hilo.join().unwrap();
}

#[tokio::test]
async fn respuesta_exitosa_invalida_no_se_convierte_en_perfil() {
    for cuerpo in ["no-json", r#"{"token":"no-es-un-perfil"}"#] {
        let (url, hilo) = servidor(200, cuerpo);
        let resultado = ClienteAutenticacionHttp::new(url)
            .obtener_perfil("token-qa")
            .await;
        assert!(resultado.is_err());
        let error = resultado.err().unwrap();
        assert_eq!(error.status().unwrap().as_u16(), 200);
        assert_eq!(
            error.mensaje_ui(),
            "El servidor devolvió una respuesta inválida."
        );
        hilo.join().unwrap();
    }
}

#[tokio::test]
async fn rechazo_bearer_de_fastapi_conserva_401_para_cerrar_sesion_local() {
    let (url, hilo) = servidor(401, r#"{"detail":"Token inválido o vencido."}"#);
    let resultado = ClienteAutenticacionHttp::new(url)
        .obtener_perfil("token-qa")
        .await;
    assert!(resultado.is_err());
    assert_eq!(resultado.err().unwrap().status().unwrap().as_u16(), 401);
    hilo.join().unwrap();
}

#[tokio::test]
async fn servidor_apagado_es_error_de_red_y_no_sesion_exitosa() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let resultado = ClienteAutenticacionHttp::new(url)
        .obtener_perfil("token-qa")
        .await;
    assert!(resultado.is_err());
    let error = resultado.err().unwrap();
    assert!(error.status().is_none());
    assert_eq!(
        error.mensaje_ui(),
        "No se pudo conectar con el servidor. Verifica tu conexión."
    );
}
