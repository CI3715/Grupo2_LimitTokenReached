//! Formato plano del libro: cabecera de 3 líneas y un evento por línea.
//!
//! ```text
//! VERSION=1.0
//! FECHA_EXPORTACION=...
//! TOTAL_EVENTOS=...
//! TIPO|ID_EVENTO|TIMESTAMP|DEVICE_ID|DATOS_JSON
//! ```
//!
//! El archivo se valida por completo antes de importarse. Antes de escribir se
//! eliminan credenciales/tokens y se enmascaran los JWT que aparezcan como texto.

use crate::repositorio_eventos_sqlite::canonizar_json;
use aplicacion::puertos::{LibroDeserializado, SerializadorLibro};
use dominio::libro::EventoLibro;
use serde_json::Value;

/// Versión del formato plano documentado en `docs/formato-exportacion.md`.
pub const VERSION_LIBRO: &str = "1.0";

const CLAVES_SENSIBLES: &[&str] = &[
    "password",
    "password_hash",
    "contrasena",
    "contraseña",
    "passphrase",
    "frase",
    "token",
    "secret",
    "secreto",
    "credencial",
    "credenciales",
    "credential",
    "credentials",
    "clave_privada",
    "authorization",
    "api_key",
    "apikey",
    "jwt",
    "session",
    "sesion",
];

fn es_clave_sensible(clave: &str) -> bool {
    let normalizada = clave.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    CLAVES_SENSIBLES
        .iter()
        .any(|sensible| normalizada.contains(sensible))
}

/// Enmascara tokens JWT y cabeceras `Bearer` que aparezcan dentro de un valor de texto.
fn mascarar_texto(texto: &str) -> String {
    const FILTRADO: &str = "[FILTRADO]";
    let minusculas = texto.to_ascii_lowercase();
    if minusculas.starts_with("bearer ")
        || (texto.starts_with("eyJ") && texto.matches('.').count() == 2)
    {
        FILTRADO.to_string()
    } else {
        texto.to_string()
    }
}

/// Elimina de forma recursiva credenciales, tokens y JWT de cualquier payload.
fn filtrar_credenciales(valor: &Value) -> Value {
    match valor {
        Value::Object(mapa) => Value::Object(
            mapa.iter()
                .filter(|(clave, _)| !es_clave_sensible(clave))
                .map(|(clave, valor)| (clave.clone(), filtrar_credenciales(valor)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(filtrar_credenciales).collect()),
        Value::String(texto) => Value::String(mascarar_texto(texto)),
        otro => otro.clone(),
    }
}

fn valor_de_linea<'a>(linea: &'a str, clave: &str) -> Result<&'a str, String> {
    linea
        .strip_prefix(&format!("{clave}="))
        .filter(|valor| !valor.is_empty())
        .ok_or_else(|| format!("Falta {clave} en el encabezado del libro."))
}

pub struct SerializadorLibroPlano;

impl SerializadorLibroPlano {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SerializadorLibroPlano {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializadorLibro for SerializadorLibroPlano {
    fn serializar_plano(
        &self,
        device_id: &str,
        exportado_en: &str,
        eventos: &[EventoLibro],
    ) -> Result<Vec<u8>, String> {
        let mut lineas = Vec::with_capacity(eventos.len() + 3);
        lineas.push(format!("VERSION={VERSION_LIBRO}"));
        lineas.push(format!("FECHA_EXPORTACION={exportado_en}"));
        lineas.push(format!("TOTAL_EVENTOS={}", eventos.len()));

        for evento in eventos {
            for campo in [
                &evento.tipo,
                &evento.id,
                &evento.registrado_en,
                &evento.dispositivo,
            ] {
                if campo.is_empty()
                    || campo.contains('|')
                    || campo.contains('\n')
                    || campo.contains('\r')
                {
                    return Err(format!(
                        "El evento {} tiene un campo con formato inválido.",
                        evento.id
                    ));
                }
            }

            let crudo: Value = serde_json::from_str(&evento.datos_canonicos)
                .map_err(|_| format!("JSON inválido en el evento {}.", evento.id))?;
            let datos_limpios = canonizar_json(&filtrar_credenciales(&crudo));
            let datos_json = serde_json::to_string(&datos_limpios)
                .map_err(|_| format!("No se pudo serializar el evento {}.", evento.id))?;

            lineas.push(format!(
                "{}|{}|{}|{}|{}",
                evento.tipo, evento.id, evento.registrado_en, evento.dispositivo, datos_json
            ));
        }

        // `device_id` se conserva como parametro de la API para quien exporta,
        // pero el encabezado obligatorio tiene exactamente 3 lineas.
        let _ = device_id;
        Ok(format!("{}\n", lineas.join("\n")).into_bytes())
    }

    fn deserializar_plano(&self, contenido: &[u8]) -> Result<LibroDeserializado, String> {
        let texto = std::str::from_utf8(contenido)
            .map_err(|_| "El archivo del libro no es texto UTF-8 válido.".to_string())?;
        let lineas: Vec<&str> = texto.lines().collect();

        if lineas.len() < 3 {
            return Err("Faltan las líneas del encabezado del libro.".to_string());
        }

        let version = valor_de_linea(lineas[0], "VERSION")?;
        if version != VERSION_LIBRO {
            return Err(format!(
                "Versión de formato no soportada: {version}. La aplicación soporta la versión {VERSION_LIBRO}."
            ));
        }
        let exportado_en = valor_de_linea(lineas[1], "FECHA_EXPORTACION")?.to_string();
        let total_eventos = valor_de_linea(lineas[2], "TOTAL_EVENTOS")?
            .parse::<usize>()
            .map_err(|_| "TOTAL_EVENTOS no es un número válido.".to_string())?;

        let cuerpo = &lineas[3..];
        if cuerpo.len() != total_eventos {
            return Err(format!(
                "El encabezado anuncia {total_eventos} eventos pero hay {}.",
                cuerpo.len()
            ));
        }

        let mut eventos = Vec::with_capacity(total_eventos);
        for (indice, linea) in cuerpo.iter().enumerate() {
            let partes: Vec<&str> = linea.splitn(5, '|').collect();
            if partes.len() != 5 || partes[..4].iter().any(|parte| parte.is_empty()) {
                return Err(format!(
                    "El evento {} de la línea {} no tiene 5 campos.",
                    indice + 1,
                    indice + 4
                ));
            }

            let crudo: Value = serde_json::from_str(partes[4])
                .map_err(|_| format!("JSON inválido en el evento {}.", indice + 1))?;
            let datos_limpios = canonizar_json(&filtrar_credenciales(&crudo));
            let datos_canonicos = serde_json::to_string(&datos_limpios)
                .map_err(|_| format!("No se pudo serializar el evento {}.", indice + 1))?;

            eventos.push(EventoLibro {
                tipo: partes[0].to_string(),
                id: partes[1].to_string(),
                registrado_en: partes[2].to_string(),
                dispositivo: partes[3].to_string(),
                datos_canonicos,
            });
        }

        Ok(LibroDeserializado {
            version: version.to_string(),
            exportado_en,
            eventos,
        })
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn evento() -> EventoLibro {
        EventoLibro {
            id: "ev-1".to_string(),
            tipo: "CuentaCreada".to_string(),
            registrado_en: "2026-11-26T12:00:00Z".to_string(),
            dispositivo: "disp-1".to_string(),
            datos_canonicos: r#"{"nombre":"Efectivo","token":"secreto-jwt","contrasena":"123"}"#
                .to_string(),
        }
    }

    #[test]
    fn serializa_tres_lineas_de_cabecera_y_filtra_credenciales_y_jwt() {
        let serializador = SerializadorLibroPlano::new();
        let mut ev = evento();
        ev.datos_canonicos = r#"{"nota":"Bearer abc.def.ghi","password":"123"}"#.to_string();

        let bytes = serializador
            .serializar_plano("disp-1", "2026-11-26T12:05:00Z", &[ev])
            .unwrap();
        let texto = String::from_utf8(bytes.clone()).unwrap();
        let lineas: Vec<&str> = texto.lines().collect();

        assert_eq!(lineas.len(), 4);
        assert_eq!(lineas[0], "VERSION=1.0");
        assert_eq!(lineas[1], "FECHA_EXPORTACION=2026-11-26T12:05:00Z");
        assert_eq!(lineas[2], "TOTAL_EVENTOS=1");
        assert_eq!(lineas[3].splitn(5, '|').count(), 5);
        assert!(!texto.contains("123"));
        assert!(!texto.contains("abc.def.ghi"));
        assert!(texto.contains("[FILTRADO]"));

        let leido = serializador.deserializar_plano(&bytes).unwrap();
        assert_eq!(leido.version, "1.0");
        assert_eq!(leido.eventos.len(), 1);
        assert_eq!(leido.eventos[0].id, "ev-1");
    }

    #[test]
    fn rechaza_version_no_soportada_y_archivo_invalido_completo() {
        let serializador = SerializadorLibroPlano::new();
        let bytes = serializador
            .serializar_plano("disp-1", "2026-11-26T12:05:00Z", &[evento()])
            .unwrap();

        let mut texto = String::from_utf8(bytes).unwrap();
        let error_version = serializador
            .deserializar_plano(texto.replace("VERSION=1.0", "VERSION=9.9.9").as_bytes())
            .unwrap_err();
        assert!(error_version.contains("Versión de formato no soportada: 9.9.9"));
        assert!(error_version.contains("1.0"));

        texto = texto.replace("CuentaCreada|ev-1", "CuentaCreada|");
        let error_linea = serializador
            .deserializar_plano(texto.as_bytes())
            .unwrap_err();
        assert!(error_linea.contains("no tiene 5 campos"));
    }
}
