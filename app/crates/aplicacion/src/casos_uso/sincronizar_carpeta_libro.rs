use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;

use crate::puertos::{
    CifradorRespaldo, Reloj, ReporteSincronizacionLibro, RepositorioEventos, SerializadorLibro,
};

const ERROR_CARPETA: &str = "No se pudo llegar a la carpeta; se reintentará.";
static CONTADOR_LOTES: AtomicU64 = AtomicU64::new(0);

fn recoger_lotes(directorio: &Path, salida: &mut Vec<PathBuf>) -> Result<(), String> {
    let entradas = fs::read_dir(directorio).map_err(|_| ERROR_CARPETA.to_string())?;
    for entrada in entradas {
        let entrada = entrada.map_err(|_| ERROR_CARPETA.to_string())?;
        let ruta = entrada.path();
        if ruta.is_dir() {
            recoger_lotes(&ruta, salida)?;
        } else if ruta
            .extension()
            .is_some_and(|extension| extension == "ccbak")
        {
            salida.push(ruta);
        }
    }
    Ok(())
}

fn ruta_temporal_atomica(destino: &Path) -> PathBuf {
    let mut temp = destino.as_os_str().to_os_string();
    temp.push(".tmp");
    PathBuf::from(temp)
}

fn escribir_atomico(destino: &Path, bytes: &[u8]) -> Result<(), String> {
    let ruta_tmp = ruta_temporal_atomica(destino);
    fs::write(&ruta_tmp, bytes).map_err(|_| ERROR_CARPETA.to_string())?;
    fs::rename(&ruta_tmp, destino).map_err(|_| {
        let _ = fs::remove_file(&ruta_tmp);
        ERROR_CARPETA.to_string()
    })
}

/// Sincroniza el libro con una carpeta compartida usando lotes cifrados e inmutables.
///
/// 1. Trae y descifra los lotes `*.ccbak` de todos los dispositivos.
/// 2. Fusiona los eventos recibidos con la regla de la sección 2.1.
/// 3. Escribe un lote nuevo, solo con los eventos locales aún no enviados, dentro
///    de la carpeta propia del dispositivo. El nombre es único y nunca se reescribe.
pub fn ejecutar(
    ruta_carpeta: &Path,
    frase_paso: &str,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
    reloj: &dyn Reloj,
) -> Result<ReporteSincronizacionLibro, String> {
    fs::create_dir_all(ruta_carpeta).map_err(|_| ERROR_CARPETA.to_string())?;

    let device_id = repo_eventos.obtener_device_id()?;
    let carpeta_propia = ruta_carpeta.join(&device_id);
    fs::create_dir_all(&carpeta_propia).map_err(|_| ERROR_CARPETA.to_string())?;

    let mut lotes = Vec::new();
    recoger_lotes(ruta_carpeta, &mut lotes)?;
    lotes.sort();

    let mut eventos_entrantes = Vec::new();
    let mut ya_enviados = std::collections::HashSet::new();

    for ruta in lotes {
        let cifrado = fs::read(&ruta).map_err(|_| ERROR_CARPETA.to_string())?;
        let plano = cifrador.descifrar(&cifrado, frase_paso)?;
        let libro = serializador.deserializar_plano(&plano)?;

        if ruta.starts_with(&carpeta_propia) {
            ya_enviados.extend(libro.eventos.iter().map(|evento| evento.id.clone()));
        }
        eventos_entrantes.extend(libro.eventos);
    }

    let resumen = repo_eventos.fusionar_lote_transaccional(&eventos_entrantes)?;
    let eventos_locales = repo_eventos.listar_eventos_ordenados()?;
    let para_enviar: Vec<_> = eventos_locales
        .into_iter()
        .filter(|evento| !ya_enviados.contains(&evento.id))
        .collect();
    let eventos_enviados = para_enviar.len();

    if !para_enviar.is_empty() {
        let ahora = reloj.ahora();
        let plano = serializador.serializar_plano(&device_id, &ahora, &para_enviar)?;
        let cifrado = cifrador.cifrar(&plano, frase_paso)?;
        let nanos = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let secuencia = CONTADOR_LOTES.fetch_add(1, Ordering::Relaxed);
        let destino = carpeta_propia.join(format!("lote-{nanos}-{secuencia}.ccbak"));
        escribir_atomico(&destino, &cifrado)?;
    }

    Ok(ReporteSincronizacionLibro {
        eventos_enviados,
        eventos_recibidos: resumen.insertados,
        eventos_ya_existentes: resumen.ignorados_duplicados,
        rechazados: resumen.rechazados,
        fecha_ultima_sincronizacion: reloj.ahora(),
    })
}
