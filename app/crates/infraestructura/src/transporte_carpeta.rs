//! Adaptador de `TransporteSincronizacion` basado en una carpeta compartida.
//!
//! Los lotes cifrados viven en subcarpetas por dispositivo y son inmutables:
//! cada publicación crea un archivo nuevo con un nombre único.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use aplicacion::puertos::{LoteRecibido, TransporteSincronizacion};
use chrono::Utc;

const ERROR_CARPETA: &str = "No se pudo llegar a la carpeta; se reintentará.";
static CONTADOR_LOTES: AtomicU64 = AtomicU64::new(0);

/// Sincroniza los lotes del libro con una carpeta compartida por el usuario.
#[derive(Debug, Clone)]
pub struct TransporteCarpeta {
    ruta: PathBuf,
}

impl TransporteCarpeta {
    pub fn new(ruta: impl Into<PathBuf>) -> Self {
        Self { ruta: ruta.into() }
    }
}

impl TransporteSincronizacion for TransporteCarpeta {
    fn leer_lotes(&self) -> Result<Vec<LoteRecibido>, String> {
        fs::create_dir_all(&self.ruta).map_err(|_| ERROR_CARPETA.to_string())?;

        let mut rutas = Vec::new();
        recoger_rutas(&self.ruta, &mut rutas)?;
        rutas.sort();

        rutas
            .into_iter()
            .map(|ruta| {
                Ok(LoteRecibido {
                    dispositivo: dispositivo_de_ruta(&self.ruta, &ruta),
                    contenido: fs::read(&ruta).map_err(|_| ERROR_CARPETA.to_string())?,
                })
            })
            .collect()
    }

    fn publicar_lote(&self, dispositivo: &str, contenido: &[u8]) -> Result<(), String> {
        let nombre = Path::new(dispositivo)
            .file_name()
            .and_then(|valor| valor.to_str())
            .filter(|valor| !valor.is_empty())
            .unwrap_or("desconocido");
        let carpeta = self.ruta.join(nombre);
        fs::create_dir_all(&carpeta).map_err(|_| ERROR_CARPETA.to_string())?;

        let nanos = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let secuencia = CONTADOR_LOTES.fetch_add(1, Ordering::Relaxed);
        let destino = carpeta.join(format!("lote-{nanos}-{secuencia}.ccbak"));
        escribir_atomico(&destino, contenido)
    }
}

fn recoger_rutas(directorio: &Path, salida: &mut Vec<PathBuf>) -> Result<(), String> {
    let entradas = fs::read_dir(directorio).map_err(|_| ERROR_CARPETA.to_string())?;
    for entrada in entradas {
        let entrada = entrada.map_err(|_| ERROR_CARPETA.to_string())?;
        let ruta = entrada.path();
        if ruta.is_dir() {
            recoger_rutas(&ruta, salida)?;
        } else if ruta
            .extension()
            .is_some_and(|extension| extension == "ccbak")
        {
            salida.push(ruta);
        }
    }
    Ok(())
}

fn dispositivo_de_ruta(raiz: &Path, ruta: &Path) -> String {
    let relativo = ruta.strip_prefix(raiz).unwrap_or(ruta);
    let mut componentes = relativo.components();
    match (componentes.next(), componentes.next()) {
        (Some(primero), Some(_)) => primero.as_os_str().to_string_lossy().into_owned(),
        _ => String::new(),
    }
}

fn escribir_atomico(destino: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut temp = destino.as_os_str().to_os_string();
    temp.push(".tmp");
    let temporal = PathBuf::from(temp);
    fs::write(&temporal, bytes).map_err(|_| ERROR_CARPETA.to_string())?;
    fs::rename(&temporal, destino).map_err(|_| {
        let _ = fs::remove_file(&temporal);
        ERROR_CARPETA.to_string()
    })
}
