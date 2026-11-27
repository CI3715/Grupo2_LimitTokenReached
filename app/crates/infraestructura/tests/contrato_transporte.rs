use std::process::Command;

use aplicacion::puertos::TransporteSincronizacion;
use infraestructura::transporte_carpeta::TransporteCarpeta;
use infraestructura::transporte_git::TransporteGit;
use infraestructura::transporte_memoria::AdaptadorMemoria;

// Funciones de Contrato (RNF-14, Sección 2.5)

/// Prueba de contrato: publicar un lote y leerlo debe devolver el contenido exacto
fn contrato_publicar_y_leer(transporte: &dyn TransporteSincronizacion) {
    let contenido = b"datos cifrados de prueba";
    transporte.publicar_lote("test-device", contenido).unwrap();

    let lotes = transporte.leer_lotes().unwrap();
    let lote_encontrado = lotes.iter().find(|l| l.dispositivo == "test-device");

    assert!(
        lote_encontrado.is_some(),
        "debe existir el lote del dispositivo de prueba"
    );
    assert_eq!(
        lote_encontrado.unwrap().contenido,
        contenido,
        "el contenido no debe corromperse"
    );
}

/// Prueba de contrato: múltiples dispositivos pueden publicar lotes independientes
fn contrato_multiples_dispositivos(transporte: &dyn TransporteSincronizacion) {
    transporte.publicar_lote("disp-a", b"lote-a").unwrap();
    transporte.publicar_lote("disp-b", b"lote-b").unwrap();

    let lotes = transporte.leer_lotes().unwrap();
    let lote_a = lotes
        .iter()
        .find(|lote| lote.dispositivo == "disp-a")
        .expect("debe existir el lote de disp-a");
    let lote_b = lotes
        .iter()
        .find(|lote| lote.dispositivo == "disp-b")
        .expect("debe existir el lote de disp-b");
    assert_eq!(lote_a.contenido, b"lote-a");
    assert_eq!(lote_b.contenido, b"lote-b");
}

// Tests para los 3 Adaptadores

#[test]
fn adaptador_memoria_cumple_el_contrato() {
    let transporte = AdaptadorMemoria::nuevo();
    contrato_publicar_y_leer(&transporte);
    contrato_multiples_dispositivos(&transporte);
}

#[test]
fn adaptador_carpeta_cumple_el_contrato() {
    let ruta_temp = std::env::temp_dir().join("cc-test-carpeta-contrato");
    let _ = std::fs::remove_dir_all(&ruta_temp);

    let transporte = TransporteCarpeta::new(&ruta_temp);
    contrato_publicar_y_leer(&transporte);
    contrato_multiples_dispositivos(&transporte);

    let _ = std::fs::remove_dir_all(&ruta_temp);
}

#[test]
fn adaptador_git_cumple_el_contrato() {
    // Creamos un directorio temporal para el repositorio bare (remoto simulado)
    let ruta_temp = std::env::temp_dir().join("cc-test-git-contrato");
    let _ = std::fs::remove_dir_all(&ruta_temp);
    std::fs::create_dir_all(&ruta_temp).unwrap();

    let ruta_remoto = ruta_temp.join("remoto.git");
    let ruta_clon = ruta_temp.join("clon");

    // Inicializamos un repositorio bare local (simula el remoto de GitHub)
    Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg("--quiet")
        .arg(&ruta_remoto)
        .output()
        .expect("git init bare falló");

    let url_remoto = ruta_remoto.to_string_lossy().into_owned();
    let transporte = TransporteGit::new(&url_remoto, &ruta_clon);

    // El adaptador de Git necesita clonar el remoto antes de operar
    transporte.asegurar_clon().unwrap();

    // Configuramos la identidad de Git para que el commit de prueba no falle
    Command::new("git")
        .arg("-C")
        .arg(&ruta_clon)
        .args(&["config", "user.email", "test@cuentasclaras.local"])
        .output()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&ruta_clon)
        .args(&["config", "user.name", "Test QA"])
        .output()
        .unwrap();

    // Ejecutamos las pruebas de contrato
    contrato_publicar_y_leer(&transporte);
    contrato_multiples_dispositivos(&transporte);

    // Limpiamos la carpeta temporal
    let _ = std::fs::remove_dir_all(&ruta_temp);
}
