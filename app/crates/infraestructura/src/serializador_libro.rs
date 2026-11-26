use std::io::{BufRead, BufReader};

use aplicacion::puertos::{LibroDeserializado, SerializadorLibro};
use dominio::libro::{EventoLibro, EventoRechazado};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::repositorio_eventos_sqlite::{canonizar_json, canonizar_json_str};

const FORMATO_LIBRO: &str = "cuentas-claras-libro";
const VERSION_LIBRO: u32 = 1;

/// Claves sensibles que jamás deben exportarse en un respaldo o carpeta sincronizada.
const CLAVES_SENSIBLES: &[&str] = &[
    "token",
    "refresh_token",
    "access_token",
    "contrasena",
    "password",
    "secret",
    "secreto",
    "clave_privada",
    "authorization",
    "api_key",
];

#[derive(Debug, Serialize, Deserialize)]
struct CabeceraLibroNdjson {
    formato: String,
    version: u32,
    exportado_en: String,
    device_id: String,
    total_eventos: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct LineaEventoNdjson {
    id: String,
    tipo: String,
    registrado_en: String,
    dispositivo: String,
    datos: Value,
}

/// Elimina recursivamente cualquier campo que coincida con credenciales o tokens.
fn filtrar_credenciales(valor: &Value) -> Value {
    match valor {
        Value::Object(mapa) => {
            let mut limpio = serde_json::Map::new();
            for (k, v) in mapa {
                let clave_normalizada = k.trim().to_ascii_lowercase();
                if CLAVES_SENSIBLES.contains(&clave_normalizada.as_str()) {
                    continue;
                }
                limpio.insert(k.clone(), filtrar_credenciales(v));
            }
            Value::Object(limpio)
        }
        Value::Array(items) => Value::Array(items.iter().map(filtrar_credenciales).collect()),
        otro => otro.clone(),
    }
}

pub struct SerializadorLibroNdjson;

impl SerializadorLibroNdjson {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SerializadorLibroNdjson {
    fn default() -> Self {
        Self::new()
    }
}

impl SerializadorLibro for SerializadorLibroNdjson {
    fn serializar_plano(
        &self,
        device_id: &str,
        exportado_en: &str,
        eventos: &[EventoLibro],
    ) -> Result<Vec<u8>, String> {
        let mut salida = String::new();

        // Línea 1: Cabecera del archivo
        let cabecera = CabeceraLibroNdjson {
            formato: FORMATO_LIBRO.to_string(),
            version: VERSION_LIBRO,
            exportado_en: exportado_en.to_string(),
            device_id: device_id.to_string(),
            total_eventos: eventos.len(),
        };
        let linea_cabecera = serde_json::to_string(&cabecera)
            .map_err(|e| format!("Error al serializar cabecera del libro: {e}"))?;
        salida.push_str(&linea_cabecera);
        salida.push('\n');

        // Líneas 2..N+1: 1 evento compacto por línea, filtrado y canonizado
        for ev in eventos {
            let valor_crudo: Value = serde_json::from_str(&ev.datos_canonicos)
                .map_err(|e| format!("Evento {} contiene JSON inválido: {e}", ev.id))?;
            let datos_limpios = canonizar_json(&filtrar_credenciales(&valor_crudo));

            let registro = LineaEventoNdjson {
                id: ev.id.clone(),
                tipo: ev.tipo.clone(),
                registrado_en: ev.registrado_en.clone(),
                dispositivo: ev.dispositivo.clone(),
                datos: datos_limpios,
            };

            let linea = serde_json::to_string(&registro)
                .map_err(|e| format!("Error al serializar evento {}: {e}", ev.id))?;
            salida.push_str(&linea);
            salida.push('\n');
        }

        Ok(salida.into_bytes())
    }

    fn deserializar_plano(&self, contenido: &[u8]) -> Result<LibroDeserializado, String> {
        let lector = BufReader::new(contenido);
        let mut lineas = lector.lines();

        // 1. Leer y validar la cabecera obligatoria en la línea 1
        let primera_linea = lineas
            .next()
            .ok_or_else(|| "El archivo del libro está vacío.".to_string())?
            .map_err(|e| format!("No se pudo leer la cabecera del libro: {e}"))?;

        let cabecera: CabeceraLibroNdjson = serde_json::from_str(primera_linea.trim())
            .map_err(|e| format!("Cabecera de respaldo inválida: {e}"))?;

        if cabecera.formato != FORMATO_LIBRO {
            return Err(format!(
                "Formato de archivo no reconocido: '{}'",
                cabecera.formato
            ));
        }
        if cabecera.version != VERSION_LIBRO {
            return Err(format!(
                "Versión de formato no soportada: {}",
                cabecera.version
            ));
        }

        // 2. Leer cada evento línea por línea sin abortar ante líneas corruptas individuales
        let mut eventos = Vec::new();
        let mut lineas_invalidas = Vec::new();

        for (indice, resultado_linea) in lineas.enumerate() {
            let numero_linea = indice + 2;
            let linea = match resultado_linea {
                Ok(texto) => texto,
                Err(e) => {
                    lineas_invalidas.push(EventoRechazado {
                        id: format!("linea-{numero_linea}"),
                        motivo: format!("Error de lectura UTF-8 en línea {numero_linea}: {e}"),
                    });
                    continue;
                }
            };

            let recortada = linea.trim();
            if recortada.is_empty() {
                continue;
            }

            let registro: LineaEventoNdjson = match serde_json::from_str(recortada) {
                Ok(parsed) => parsed,
                Err(e) => {
                    lineas_invalidas.push(EventoRechazado {
                        id: format!("linea-{numero_linea}"),
                        motivo: format!("JSON inválido en línea {numero_linea}: {e}"),
                    });
                    continue;
                }
            };

            if registro.id.trim().is_empty()
                || registro.tipo.trim().is_empty()
                || registro.registrado_en.trim().is_empty()
            {
                lineas_invalidas.push(EventoRechazado {
                    id: if registro.id.trim().is_empty() {
                        format!("linea-{numero_linea}")
                    } else {
                        registro.id
                    },
                    motivo: format!("Faltan campos obligatorios en línea {numero_linea}"),
                });
                continue;
            }

            let datos_limpios = canonizar_json(&filtrar_credenciales(&registro.datos));
            let datos_canonicos = match serde_json::to_string(&datos_limpios)
                .and_then(|s| canonizar_json_str(&s).map_err(serde::ser::Error::custom))
            {
                Ok(s) => s,
                Err(e) => {
                    lineas_invalidas.push(EventoRechazado {
                        id: registro.id,
                        motivo: format!("Payload inválido en línea {numero_linea}: {e}"),
                    });
                    continue;
                }
            };

            eventos.push(EventoLibro {
                id: registro.id,
                tipo: registro.tipo,
                registrado_en: registro.registrado_en,
                dispositivo: registro.dispositivo,
                datos_canonicos,
            });
        }

        Ok(LibroDeserializado {
            device_id_origen: cabecera.device_id,
            exportado_en: cabecera.exportado_en,
            eventos,
            lineas_invalidas,
        })
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn serializa_filtra_credenciales_y_deserializa_ndjson() {
        let serializador = SerializadorLibroNdjson::new();
        let ev = EventoLibro {
            id: "ev-1".to_string(),
            tipo: "CuentaCreada".to_string(),
            registrado_en: "2026-11-26T12:00:00Z".to_string(),
            dispositivo: "disp-1".to_string(),
            datos_canonicos: r#"{"nombre":"Efectivo","token":"secreto-jwt","contrasena":"123"}"#
                .to_string(),
        };

        let bytes = serializador
            .serializar_plano("disp-1", "2026-11-26T12:05:00Z", &[ev])
            .unwrap();
        let texto = String::from_utf8(bytes.clone()).unwrap();

        assert_eq!(texto.lines().count(), 2);
        assert!(!texto.contains("secreto-jwt"));
        assert!(!texto.contains("contrasena"));

        let leido = serializador.deserializar_plano(&bytes).unwrap();
        assert_eq!(leido.device_id_origen, "disp-1");
        assert_eq!(leido.eventos.len(), 1);
        assert_eq!(leido.eventos[0].datos_canonicos, r#"{"nombre":"Efectivo"}"#);
        assert!(leido.lineas_invalidas.is_empty());
    }
}