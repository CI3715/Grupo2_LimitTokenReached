//! Pruebas del cifrado y la seguridad del respaldo.
//!
//! Usan el cifrador de verdad (Argon2id + AES-256-GCM), que en pruebas tarda unos segundos por
//! exportación o importación. Para no esperar de más, las pruebas comparten una exportación
//! cifrada y los rechazos que ocurren antes de derivar la clave (versión, parámetros, formato)
//! no la derivan.
//!
//! Los datos salen de `tests/comun` y la frase es siempre la misma.

mod comun;

use aplicacion::casos_uso::{exportar_respaldo_cifrado, importar_respaldo_cifrado};
use aplicacion::puertos::{MAGIA_RESPALDO_CIFRADO, Reloj};
use base64::{Engine, engine::general_purpose::STANDARD};
use comun::*;
use dominio::libro::ResumenFusion;
use infraestructura::{CifradorRespaldoAesGcm, SerializadorLibroPlano};
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::OnceLock;

const FRASE: &str = "frase-de-paso-segura-2026";
const ERROR_DESCIFRADO: &str = "Frase de paso incorrecta o archivo dañado.";

fn d(texto: &str) -> Decimal {
    texto.parse().unwrap()
}

struct RelojFijo;

impl Reloj for RelojFijo {
    fn ahora(&self) -> String {
        "2026-11-26T12:00:00Z".to_string()
    }
}

fn id_extra(n: u128) -> String {
    uuid::Uuid::from_u128(0x2600_0000_0000_4000_8000_0002_0000_0000 + n).to_string()
}

fn evento_tasa() -> EventoPrueba {
    EventoPrueba {
        id: id_extra(1),
        tipo: "TasaRegistrada".to_string(),
        registrado_en: "2026-11-12T10:00:00.000000000+00:00".to_string(),
        dispositivo: DISPOSITIVO_LAPTOP.to_string(),
        datos: json!({
            "tasa_id": "9001",
            "moneda_origen": "USD",
            "moneda_destino": "VES",
            "valor": "36.5",
            "fecha": "2026-11-12",
            "fuente": "BCV",
        }),
    }
}

/// Los 44 eventos del escenario 2.6 más una tasa de cambio (45).
fn libro_de_prueba() -> Vec<EventoPrueba> {
    let mut libro = Escenario26::nuevo().todos();
    libro.push(evento_tasa());
    libro
}

/// Base de 40 eventos más un gasto con una nota "secreta" y credenciales falsas.
fn libro_con_secretos() -> Vec<EventoPrueba> {
    let mut libro = Escenario26::nuevo().base.clone();
    libro.push(EventoPrueba {
        id: id_extra(2),
        tipo: "MovimientoRegistrado".to_string(),
        registrado_en: "2026-11-12T11:00:00.000000000+00:00".to_string(),
        dispositivo: DISPOSITIVO_LAPTOP.to_string(),
        datos: json!({
            "id": id_extra(3),
            "cuenta_id": CUENTA_EFECTIVO,
            "tipo": "Gasto",
            "monto": "7.77",
            "fecha": "2026-11-12",
            "categoria_id": "00000000-0000-0000-0000-000000000001",
            "nota": "NOTA-SECRETA-CENA-FAMILIAR",
            "token": "FAKE-TOKEN-ABC123",
            "jwt_sesion": "eyJhbGciOiJIUzI1NiJ9.cGF5bG9hZA.c2ln",
            "referencia": "eyJhbGciOiJIUzI1NiJ9.cGF5bG9hZA.c2ln",
            "cabecera": "Bearer abc123def456",
            "extra": { "password": "FAKE-PASSWORD-XYZ" },
        }),
    });
    libro
}

fn exportar_cifrado(origen: &Dispositivo, frase: &str) -> Vec<u8> {
    let archivo = ArchivoTemporal::nuevo("cifrado", "ccbak");
    exportar_respaldo_cifrado::ejecutar(
        &archivo.0,
        frase,
        &origen.repo,
        &SerializadorLibroPlano::new(),
        &CifradorRespaldoAesGcm::new(),
        &RelojFijo,
    )
    .unwrap();
    std::fs::read(&archivo.0).unwrap()
}

fn exportar_plano(origen: &Dispositivo) -> Vec<u8> {
    let archivo = ArchivoTemporal::nuevo("plano", "txt");
    exportar_respaldo_cifrado::ejecutar_plano(
        &archivo.0,
        &origen.repo,
        &SerializadorLibroPlano::new(),
        &RelojFijo,
    )
    .unwrap();
    std::fs::read(&archivo.0).unwrap()
}

/// Exportación cifrada del libro de prueba, se hace una sola vez para todas las pruebas.
fn exportacion_compartida() -> &'static [u8] {
    static EXPORTACION: OnceLock<Vec<u8>> = OnceLock::new();
    EXPORTACION.get_or_init(|| {
        let origen = Dispositivo::con_eventos(&libro_de_prueba());
        exportar_cifrado(&origen, FRASE)
    })
}

fn importar(destino: &Dispositivo, bytes: &[u8], frase: &str) -> Result<ResumenFusion, String> {
    let archivo = ArchivoTemporal::nuevo("importar", "ccbak");
    std::fs::write(&archivo.0, bytes).unwrap();
    importar_respaldo_cifrado::ejecutar(
        &archivo.0,
        frase,
        &destino.repo,
        &SerializadorLibroPlano::new(),
        &CifradorRespaldoAesGcm::new(),
    )
}

fn texto_de(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn valor_de(texto: &str, clave: &str) -> String {
    texto
        .lines()
        .find_map(|linea| linea.strip_prefix(&format!("{clave}=")))
        .unwrap_or_else(|| panic!("no hay una línea {clave}="))
        .to_string()
}

/// Cambia el valor de una línea `CLAVE=valor` del archivo.
fn con_linea(texto: &str, clave: &str, valor: &str) -> String {
    let prefijo = format!("{clave}=");
    let lineas: Vec<String> = texto
        .lines()
        .map(|linea| {
            if linea.starts_with(&prefijo) {
                format!("{prefijo}{valor}")
            } else {
                linea.to_string()
            }
        })
        .collect();
    format!("{}\n", lineas.join("\n"))
}

fn contiene(bytes: &[u8], aguja: &str) -> bool {
    bytes
        .windows(aguja.len())
        .any(|ventana| ventana == aguja.as_bytes())
}

/// Importa el archivo en una base con datos y revisa que lo rechace sin tocarla.
fn rechaza_sin_tocar_la_base(archivo: &[u8], frase: &str, mensaje: &str, caso: &str) {
    let destino = Dispositivo::con_eventos(&Escenario26::nuevo().base);
    let antes = destino.foto();

    let error = importar(&destino, archivo, frase)
        .err()
        .unwrap_or_else(|| panic!("{caso}: se esperaba un error y se importó"));

    assert!(
        error.contains(mensaje),
        "{caso}: mensaje inesperado: {error}"
    );
    assert_eq!(destino.foto(), antes, "{caso}: la base cambió");
    assert_eq!(destino.contar_eventos(), 40, "{caso}");
    assert_eq!(destino.saldo(), d(SALDO_BASE), "{caso}");
}

fn filas_tasas(conn: &Connection) -> Vec<String> {
    let mut sentencia = conn
        .prepare(
            "SELECT tasa_id, moneda_entrada, moneda_salida, valor, fecha, fuente, eliminada
             FROM tasas ORDER BY tasa_id",
        )
        .unwrap();
    sentencia
        .query_map([], |fila| {
            Ok(format!(
                "{}|{}|{}|{}|{}|{}|{}",
                fila.get::<_, i64>(0)?,
                fila.get::<_, String>(1)?,
                fila.get::<_, String>(2)?,
                fila.get::<_, String>(3)?,
                fila.get::<_, String>(4)?,
                fila.get::<_, String>(5)?,
                fila.get::<_, i64>(6)?,
            ))
        })
        .unwrap()
        .map(|fila| fila.unwrap())
        .collect()
}

// ida y vuelta: exportar cifrado, restaurar en base limpia y comparar

#[test]
fn ida_y_vuelta_restaura_eventos_saldos_tasas_e_historial_identicos() {
    let origen = Dispositivo::con_eventos(&libro_de_prueba());
    let archivo = exportacion_compartida();
    assert!(archivo.starts_with(MAGIA_RESPALDO_CIFRADO.as_bytes()));

    // instalación nueva: base limpia
    let destino = Dispositivo::nuevo();
    let resumen = importar(&destino, archivo, FRASE).unwrap();

    assert_eq!(resumen.insertados, 45);
    assert_eq!(resumen.ignorados_duplicados, 0);
    assert!(resumen.rechazados.is_empty(), "{:?}", resumen.rechazados);
    assert_eq!(
        destino.foto(),
        origen.foto(),
        "eventos e historial deben ser idénticos"
    );
    assert_eq!(destino.saldo(), d(SALDO_CONVERGIDO));
    assert_eq!(destino.saldo_recalculado(), d(SALDO_CONVERGIDO));
    assert_eq!(
        contar_filas(&destino.lectura, "movimiento"),
        contar_filas(&origen.lectura, "movimiento")
    );
    assert_eq!(filas_tasas(&destino.lectura), filas_tasas(&origen.lectura));
    assert!(
        filas_tasas(&destino.lectura)
            .iter()
            .any(|fila| fila.starts_with("9001|USD|VES|36.5|")),
        "la tasa registrada debe restaurarse"
    );

    // la frase no queda guardada en la base
    let con_la_frase: i64 = destino
        .lectura
        .query_row(
            "SELECT COUNT(*) FROM eventos WHERE datos LIKE ?1",
            [format!("%{FRASE}%")],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(con_la_frase, 0);
    let en_meta: i64 = destino
        .lectura
        .query_row(
            "SELECT COUNT(*) FROM meta WHERE valor LIKE ?1",
            [format!("%{FRASE}%")],
            |fila| fila.get(0),
        )
        .unwrap();
    assert_eq!(en_meta, 0);
}

// casos negativos: la base no debe cambiar

#[test]
fn frase_incorrecta_se_rechaza_con_el_mensaje_exigido_y_la_base_no_cambia() {
    rechaza_sin_tocar_la_base(
        exportacion_compartida(),
        "otra-frase-equivocada-99",
        ERROR_DESCIFRADO,
        "frase incorrecta",
    );
}

#[test]
fn archivo_con_el_contenido_cifrado_alterado_se_rechaza_y_la_base_no_cambia() {
    let texto = texto_de(exportacion_compartida());
    let datos = valor_de(&texto, "DATOS");
    let mut caracteres: Vec<char> = datos.chars().collect();
    let medio = caracteres.len() / 2;
    caracteres[medio] = if caracteres[medio] == 'A' { 'B' } else { 'A' };
    let alterado = con_linea(&texto, "DATOS", &caracteres.into_iter().collect::<String>());

    rechaza_sin_tocar_la_base(
        alterado.as_bytes(),
        FRASE,
        ERROR_DESCIFRADO,
        "contenido alterado",
    );
}

#[test]
fn archivo_con_la_sal_alterada_se_rechaza_y_la_base_no_cambia() {
    let texto = texto_de(exportacion_compartida());
    let mut sal = STANDARD.decode(valor_de(&texto, "SALT")).unwrap();
    sal[0] ^= 0xFF;
    let alterado = con_linea(&texto, "SALT", &STANDARD.encode(sal));

    rechaza_sin_tocar_la_base(alterado.as_bytes(), FRASE, ERROR_DESCIFRADO, "sal alterada");
}

#[test]
fn archivo_con_el_nonce_alterado_se_rechaza_y_la_base_no_cambia() {
    let texto = texto_de(exportacion_compartida());
    let mut nonce = STANDARD.decode(valor_de(&texto, "NONCE")).unwrap();
    nonce[0] ^= 0xFF;
    let alterado = con_linea(&texto, "NONCE", &STANDARD.encode(nonce));

    rechaza_sin_tocar_la_base(
        alterado.as_bytes(),
        FRASE,
        ERROR_DESCIFRADO,
        "nonce alterado",
    );
}

#[test]
fn archivo_truncado_se_rechaza_y_la_base_no_cambia() {
    let texto = texto_de(exportacion_compartida());

    // cortado en la cabecera: ni siquiera llega a la línea DATOS
    rechaza_sin_tocar_la_base(
        &texto.as_bytes()[..60],
        FRASE,
        ERROR_DESCIFRADO,
        "truncado en la cabecera",
    );

    // cortado a la mitad del contenido cifrado (40 caracteres de base64 válidos)
    let inicio = texto.find("\nDATOS=").unwrap() + "\nDATOS=".len();
    rechaza_sin_tocar_la_base(
        &texto.as_bytes()[..inicio + 40],
        FRASE,
        ERROR_DESCIFRADO,
        "truncado en el contenido",
    );
}

#[test]
fn version_no_soportada_se_rechaza_antes_de_descifrar_y_la_base_no_cambia() {
    let texto = texto_de(exportacion_compartida());
    // comprobación: reescribir una línea sin cambiarla deja el archivo igual
    assert_eq!(con_linea(&texto, "VERSION", "1.0"), texto);

    let otra_version = con_linea(&texto, "VERSION", "9.9.9");
    rechaza_sin_tocar_la_base(
        otra_version.as_bytes(),
        FRASE,
        "Versión de formato no soportada: 9.9.9",
        "versión 9.9.9",
    );
}

#[test]
fn parametros_o_algoritmos_desconocidos_se_rechazan_sin_tocar_la_base() {
    let texto = texto_de(exportacion_compartida());
    // el archivo no puede decidir su propio costo de derivación ni su algoritmo
    for (campo, valor) in [
        ("FORMATO", "2"),
        ("CIFRADO", "NINGUNO"),
        ("KDF", "MD5"),
        ("PARAMETROS", "M=1,T=1,P=1"),
    ] {
        let alterado = con_linea(&texto, campo, valor);
        rechaza_sin_tocar_la_base(alterado.as_bytes(), FRASE, ERROR_DESCIFRADO, campo);
    }
}

#[test]
fn un_archivo_que_no_es_un_respaldo_se_rechaza_y_la_base_no_cambia() {
    for (caso, contenido) in [
        ("vacío", Vec::new()),
        (
            "texto cualquiera",
            b"hola mundo, esto no es un respaldo".to_vec(),
        ),
        ("bytes binarios", vec![0xFF, 0xFE, 0x00, 0x01, 0x80, 0x7F]),
    ] {
        let destino = Dispositivo::con_eventos(&Escenario26::nuevo().base);
        let antes = destino.foto();

        assert!(importar(&destino, &contenido, FRASE).is_err(), "{caso}");
        assert_eq!(destino.foto(), antes, "{caso}: la base cambió");
    }
}

#[test]
fn libro_plano_invalido_se_rechaza_completo_y_no_importa_nada() {
    let origen = Dispositivo::con_eventos(&libro_de_prueba());
    let plano = texto_de(&exportar_plano(&origen));
    assert!(plano.starts_with("VERSION=1.0\n"));

    // versión del libro que no se soporta
    let version = plano.replacen("VERSION=1.0", "VERSION=2.0", 1);
    rechaza_sin_tocar_la_base(
        version.as_bytes(),
        "",
        "Versión de formato no soportada",
        "libro versión 2.0",
    );

    // el encabezado dice más eventos de los que hay
    let total = plano.replacen("TOTAL_EVENTOS=45", "TOTAL_EVENTOS=46", 1);
    rechaza_sin_tocar_la_base(total.as_bytes(), "", "anuncia", "total incorrecto");

    // las 44 primeras líneas están bien y la última rota: no se importa ninguna
    let ultima_rota = plano.trim_end().trim_end_matches('}').to_string();
    rechaza_sin_tocar_la_base(
        ultima_rota.as_bytes(),
        "",
        "JSON inválido",
        "última línea rota",
    );
}

#[test]
fn una_frase_demasiado_corta_no_exporta_nada() {
    let origen = Dispositivo::con_eventos(&Escenario26::nuevo().base);
    let archivo = ArchivoTemporal::nuevo("corta", "ccbak");

    let resultado = exportar_respaldo_cifrado::ejecutar(
        &archivo.0,
        "corta",
        &origen.repo,
        &SerializadorLibroPlano::new(),
        &CifradorRespaldoAesGcm::new(),
        &RelojFijo,
    );

    let error = resultado.expect_err("una frase de 5 caracteres no debe aceptarse");
    assert!(error.contains("12"), "{error}");
    assert!(!archivo.0.exists(), "no debe quedar ningún archivo");
}

// seguridad: nada legible ni credenciales en el archivo cifrado

#[test]
fn la_cabecera_en_claro_no_revela_datos_del_libro() {
    let texto = texto_de(exportacion_compartida());
    let mut lineas = texto.lines();

    assert_eq!(lineas.next(), Some(MAGIA_RESPALDO_CIFRADO));
    let permitidas = [
        "FORMATO",
        "VERSION",
        "CIFRADO",
        "KDF",
        "PARAMETROS",
        "SALT",
        "NONCE",
        "DATOS",
    ];
    for linea in lineas {
        let (clave, _) = linea
            .split_once('=')
            .unwrap_or_else(|| panic!("línea inesperada en la cabecera: {linea}"));
        assert!(
            permitidas.contains(&clave),
            "campo no permitido en claro: {clave}"
        );
    }
    // la cantidad de eventos y la fecha van dentro del contenido cifrado
    assert!(!texto.contains("TOTAL_EVENTOS"));
    assert!(!texto.contains("FECHA_EXPORTACION"));
    // la sal y el nonce se leen bien y no están vacíos
    assert!(
        !STANDARD
            .decode(valor_de(&texto, "SALT"))
            .unwrap()
            .is_empty()
    );
    assert!(
        !STANDARD
            .decode(valor_de(&texto, "NONCE"))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn el_archivo_cifrado_no_contiene_datos_ni_credenciales_en_claro() {
    let origen = Dispositivo::con_eventos(&libro_con_secretos());
    let foto_antes = origen.foto();

    let cifrado = exportar_cifrado(&origen, FRASE);
    let contenido = STANDARD
        .decode(valor_de(&texto_de(&cifrado), "DATOS"))
        .unwrap();

    let prohibidos = [
        FRASE,
        "NOTA-SECRETA",
        "FAKE-TOKEN",
        "FAKE-PASSWORD",
        "eyJhbGci",
        "Bearer abc123",
        "Efectivo",
        "7.77",
        "151.67",
        "13.67",
        "MovimientoRegistrado",
        "CuentaCreada",
        CUENTA_EFECTIVO,
        DISPOSITIVO_LAPTOP,
    ];
    for aguja in prohibidos {
        assert!(
            !contiene(&cifrado, aguja),
            "'{aguja}' aparece en claro en el archivo cifrado"
        );
        assert!(
            !contiene(&contenido, aguja),
            "'{aguja}' aparece en claro en el contenido cifrado"
        );
    }

    // exportar no cambia el libro
    assert_eq!(origen.foto(), foto_antes);
}

#[test]
fn dos_exportaciones_del_mismo_libro_dan_archivos_distintos() {
    let origen = Dispositivo::con_eventos(&libro_de_prueba());
    let primera = texto_de(exportacion_compartida());
    let segunda = texto_de(&exportar_cifrado(&origen, FRASE));

    assert_ne!(primera, segunda);
    assert_ne!(valor_de(&primera, "SALT"), valor_de(&segunda, "SALT"));
    assert_ne!(valor_de(&primera, "NONCE"), valor_de(&segunda, "NONCE"));
    assert_ne!(valor_de(&primera, "DATOS"), valor_de(&segunda, "DATOS"));
}

#[test]
fn la_exportacion_sin_cifrar_filtra_credenciales_y_tokens() {
    let origen = Dispositivo::con_eventos(&libro_con_secretos());
    let foto_antes = origen.foto();

    let plano = texto_de(&exportar_plano(&origen));

    for aguja in [
        FRASE,
        "FAKE-TOKEN",
        "FAKE-PASSWORD",
        "eyJhbGci",
        "Bearer abc123",
        "\"token\"",
        "\"password\"",
        "\"jwt_sesion\"",
    ] {
        assert!(
            !plano.contains(aguja),
            "'{aguja}' no debería estar en el archivo sin cifrar"
        );
    }
    // lo que no es credencial se queda y los textos con forma de token se tapan
    assert!(plano.contains("NOTA-SECRETA-CENA-FAMILIAR"));
    assert!(plano.contains("[FILTRADO]"));
    assert_eq!(plano.lines().count(), 3 + 41);
    assert_eq!(
        origen.foto(),
        foto_antes,
        "exportar no debe cambiar el libro"
    );
}
