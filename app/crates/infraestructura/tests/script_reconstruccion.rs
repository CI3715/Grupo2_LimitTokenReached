//! Compara el script independiente de Python (`scripts/reconstruir_saldos.py`) con la app.
//!
//! Se arma un libro con la app, se exporta sin cifrar con la exportación real, se corre el script
//! sobre ese archivo y se comprueba que los saldos por cuenta son los mismos que tiene la app.
//! Así se demuestra que el formato de exportación basta para reconstruir los saldos con un
//! programa externo que no usa nada del código de la app.
//!
//! Necesita Python 3 instalado (`python`, `python3` o `py`).

mod comun;

use aplicacion::casos_uso::aplicar_eventos_grupo::{
    id_evento_aplicado_desde_servidor, id_evento_movimiento_desde_servidor,
    id_movimiento_desde_evento_servidor,
};
use aplicacion::casos_uso::exportar_respaldo_cifrado;
use aplicacion::puertos::Reloj;
use comun::*;
use infraestructura::SerializadorLibroPlano;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

const CUENTA_BANCO: &str = "00000000-0000-4000-8000-000000000027";
const CATEGORIA_GASTO: &str = "00000000-0000-0000-0000-000000000001";
const CATEGORIA_OTROS_GASTOS: &str = "00000000-0000-0000-0000-000000000007";
const CATEGORIA_INGRESO: &str = "00000000-0000-0000-0000-000000000008";

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
    uuid::Uuid::from_u128(0x2600_0000_0000_4000_8000_0003_0000_0000 + n).to_string()
}

fn evento(n: u128, tipo: &str, registrado_en: &str, datos: Value) -> EventoPrueba {
    EventoPrueba {
        id: id_extra(n),
        tipo: tipo.to_string(),
        registrado_en: registrado_en.to_string(),
        dispositivo: DISPOSITIVO_LAPTOP.to_string(),
        datos,
    }
}

/// El escenario 2.6 (44 eventos) más una corrección, una tasa, una segunda cuenta en bolívares
/// y un gasto de grupo, para que el script pase por todos los tipos de evento reales.
fn libro_variado() -> Vec<EventoPrueba> {
    let escenario = Escenario26::nuevo();
    let mut libro = escenario.todos();
    let gasto_a_corregir = escenario.laptop[0].datos["id"]
        .as_str()
        .unwrap()
        .to_string();

    // el gasto de 20,00 de la laptop se corrige a 15,00: variación de -5,00
    libro.push(evento(
        1,
        "MovimientoCorregido",
        "2026-11-11T09:00:00.000000000+00:00",
        json!({
            "id": id_extra(11),
            "movimiento_original_id": gasto_a_corregir,
            "cuenta_id": CUENTA_EFECTIVO,
            "tipo_original": "Gasto",
            "variacion_monto": "-5.00",
        }),
    ));
    libro.push(evento(
        2,
        "TasaRegistrada",
        "2026-11-12T10:00:00.000000000+00:00",
        json!({
            "tasa_id": "9001",
            "moneda_origen": "USD",
            "moneda_destino": "VES",
            "valor": "36.5",
            "fecha": "2026-11-12",
            "fuente": "BCV",
        }),
    ));

    // segunda cuenta, en bolívares
    libro.push(evento(
        3,
        "CuentaCreada",
        "2026-11-01T08:30:00.000000000+00:00",
        json!({
            "id": CUENTA_BANCO,
            "nombre": "Banco",
            "tipo": "banco",
            "moneda_codigo": "VES",
            "saldo_inicial": "1000.00",
            "creada_en": "2026-11-01",
        }),
    ));
    libro.push(evento(
        4,
        "MovimientoRegistrado",
        "2026-11-12T12:00:00.000000000+00:00",
        json!({
            "id": id_extra(14),
            "cuenta_id": CUENTA_BANCO,
            "tipo": "Gasto",
            "monto": "250.50",
            "fecha": "2026-11-12",
            "categoria_id": CATEGORIA_GASTO,
            "nota": Value::Null,
        }),
    ));
    libro.push(evento(
        5,
        "MovimientoRegistrado",
        "2026-11-12T13:00:00.000000000+00:00",
        json!({
            "id": id_extra(15),
            "cuenta_id": CUENTA_BANCO,
            "tipo": "Ingreso",
            "monto": "100.00",
            "fecha": "2026-11-12",
            "categoria_id": CATEGORIA_INGRESO,
            "nota": Value::Null,
        }),
    ));

    // gasto de grupo de 15,00 ya aplicado, con los ids derivados del evento del servidor
    let parte = "evento-cena";
    let movimiento = id_movimiento_desde_evento_servidor(parte).to_string();
    libro.push(EventoPrueba {
        id: id_evento_movimiento_desde_servidor(parte),
        tipo: "MovimientoRegistrado".to_string(),
        registrado_en: "2026-11-16T00:00:00Z".to_string(),
        dispositivo: "servidor".to_string(),
        datos: json!({
            "id": movimiento,
            "cuenta_id": CUENTA_EFECTIVO,
            "tipo": "Gasto",
            "monto": "15.00",
            "fecha": "2026-11-16",
            "categoria_id": CATEGORIA_OTROS_GASTOS,
            "nota": "Viaje · Cena",
        }),
    });
    libro.push(EventoPrueba {
        id: id_evento_aplicado_desde_servidor(parte),
        tipo: "GastoDeGrupoAplicado".to_string(),
        registrado_en: "2026-11-16T00:00:00Z".to_string(),
        dispositivo: "servidor".to_string(),
        datos: json!({
            "id_evento": parte,
            "movimiento_id": movimiento,
            "grupo_id": "grupo-viaje",
            "gasto_id": "gasto-cena",
        }),
    });
    libro
}

struct CuentaApp {
    id: String,
    nombre: String,
    moneda: String,
    saldo: Decimal,
}

fn cuentas_de_la_app(conn: &Connection) -> Vec<CuentaApp> {
    let mut sentencia = conn
        .prepare("SELECT id, nombre, moneda_codigo, saldo FROM cuenta ORDER BY id")
        .unwrap();
    sentencia
        .query_map([], |fila| {
            Ok(CuentaApp {
                id: fila.get(0)?,
                nombre: fila.get(1)?,
                moneda: fila.get(2)?,
                saldo: Decimal::new(fila.get::<_, i64>(3)?, 8),
            })
        })
        .unwrap()
        .map(|fila| fila.unwrap())
        .collect()
}

/// Exporta el libro sin cifrar con la exportación real de la app.
fn exportar_plano(origen: &Dispositivo) -> ArchivoTemporal {
    let archivo = ArchivoTemporal::nuevo("script", "txt");
    exportar_respaldo_cifrado::ejecutar_plano(
        &archivo.0,
        &origen.repo,
        &SerializadorLibroPlano::new(),
        &RelojFijo,
    )
    .unwrap();
    archivo
}

fn ruta_script() -> PathBuf {
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/reconstruir_saldos.py");
    assert!(
        ruta.exists(),
        "no se encontró el script en {}",
        ruta.display()
    );
    ruta
}

fn interprete() -> String {
    for candidato in ["python", "python3", "py"] {
        if let Ok(salida) = Command::new(candidato).arg("--version").output()
            && salida.status.success()
        {
            return candidato.to_string();
        }
    }
    panic!("No se encontró Python (python, python3 o py) para correr el script");
}

/// Corre el script sobre el archivo y devuelve su salida JSON.
fn ejecutar_script(archivo: &Path) -> Value {
    let salida = Command::new(interprete())
        .arg("-I")
        .arg(ruta_script())
        .arg(archivo)
        .arg("--json")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .expect("no se pudo ejecutar el script de Python");
    assert!(
        salida.status.success(),
        "el script falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    serde_json::from_slice(&salida.stdout).expect("la salida del script no es JSON")
}

fn saldo_del_script(resultado: &Value, cuenta: &str) -> Decimal {
    resultado["saldos"][cuenta]["saldo"]
        .as_str()
        .unwrap_or_else(|| panic!("el script no devolvió la cuenta {cuenta}"))
        .parse()
        .unwrap()
}

#[test]
fn el_script_reconstruye_los_mismos_saldos_que_la_app() {
    let app = Dispositivo::con_eventos(&libro_variado());
    let archivo = exportar_plano(&app);

    let resultado = ejecutar_script(&archivo.0);

    let cuentas = cuentas_de_la_app(&app.lectura);
    assert_eq!(cuentas.len(), 2);
    assert_eq!(resultado["saldos"].as_object().unwrap().len(), cuentas.len());
    for cuenta in &cuentas {
        let del_script = &resultado["saldos"][cuenta.id.as_str()];
        assert_eq!(del_script["nombre"].as_str(), Some(cuenta.nombre.as_str()));
        assert_eq!(del_script["moneda"].as_str(), Some(cuenta.moneda.as_str()));
        assert_eq!(
            saldo_del_script(&resultado, &cuenta.id),
            cuenta.saldo,
            "saldo distinto en la cuenta {}",
            cuenta.nombre
        );
    }

    // el script vio todos los eventos y no tuvo que descartar nada
    assert_eq!(
        resultado["eventos"].as_i64().unwrap(),
        app.contar_eventos(),
        "cantidad de eventos distinta"
    );
    assert_eq!(resultado["ya_existian"].as_i64(), Some(0));
    assert_eq!(resultado["grupo_duplicados"].as_i64(), Some(0));
    assert!(resultado["rechazados"].as_array().unwrap().is_empty());

    // y los números a mano: 166,67 + 5,00 (corrección) - 15,00 (gasto de grupo) y 1000 - 250,50 + 100
    assert_eq!(saldo_del_script(&resultado, CUENTA_EFECTIVO), d("156.67"));
    assert_eq!(saldo_del_script(&resultado, CUENTA_BANCO), d("849.50"));
}

#[test]
fn el_script_da_lo_mismo_con_las_lineas_del_archivo_en_cualquier_orden() {
    let app = Dispositivo::con_eventos(&libro_variado());
    let original = exportar_plano(&app);
    let referencia = ejecutar_script(&original.0)["saldos"].clone();

    let texto = std::fs::read_to_string(&original.0).unwrap();
    let lineas: Vec<&str> = texto.lines().collect();
    let (cabecera, eventos) = lineas.split_at(3);

    for semilla in 1..=3_u64 {
        let mut azar = Aleatorio::nuevo(semilla);
        let mut mezcladas = eventos.to_vec();
        azar.mezclar(&mut mezcladas);

        let archivo = ArchivoTemporal::nuevo("mezclado", "txt");
        let contenido = format!("{}\n{}\n", cabecera.join("\n"), mezcladas.join("\n"));
        std::fs::write(&archivo.0, contenido).unwrap();

        assert_eq!(
            ejecutar_script(&archivo.0)["saldos"],
            referencia,
            "semilla {semilla}"
        );
    }
}

/// El script tiene que ser independiente: solo biblioteca estándar y nunca decimales flotantes.
#[test]
fn el_script_solo_usa_biblioteca_estandar_y_nunca_float() {
    let texto = std::fs::read_to_string(ruta_script()).unwrap();
    let permitidos = [
        "__future__",
        "argparse",
        "dataclasses",
        "datetime",
        "decimal",
        "json",
        "pathlib",
        "re",
        "sys",
    ];

    let mut importados = 0;
    for linea in texto.lines().map(str::trim) {
        let modulo = if let Some(resto) = linea.strip_prefix("import ") {
            resto.split_whitespace().next()
        } else if linea.starts_with("from ") && linea.contains(" import ") {
            linea.split_whitespace().nth(1)
        } else {
            None
        };
        if let Some(modulo) = modulo {
            let raiz = modulo.split('.').next().unwrap();
            assert!(
                permitidos.contains(&raiz),
                "el script importa '{raiz}', que no es de la biblioteca estándar permitida"
            );
            importados += 1;
        }
    }
    assert!(importados > 0, "no se encontró ningún import en el script");
    assert!(!texto.contains("float("), "el script usa float");
}
