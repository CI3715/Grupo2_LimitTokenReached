use std::fs;
use std::path::{Path, PathBuf};

use dominio::libro::ResumenFusion;

use crate::puertos::{Reloj, RepositorioEventos, SerializadorLibro};

fn ruta_temporal_atomica(destino: &Path) -> PathBuf {
    let mut temp = destino.as_os_str().to_os_string();
    temp.push(".tmp");
    PathBuf::from(temp)
}

/// Sincroniza el libro local con una carpeta compartida o repositorio Git elegido por el usuario:
/// 1. Lee todos los archivos `.ndjson` presentes en la carpeta y fusiona sus eventos en SQLite.
/// 2. Escribe el archivo `<device_id>.ndjson` actualizado con el estado fusionado y filtrado.
pub fn ejecutar(
    ruta_carpeta: &Path,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    reloj: &dyn Reloj,
) -> Result<ResumenFusion, String> {
    fs::create_dir_all(ruta_carpeta)
        .map_err(|e| format!("No se pudo acceder a la carpeta de sincronización: {e}"))?;

    let mut archivos_ndjson: Vec<PathBuf> = fs::read_dir(ruta_carpeta)
        .map_err(|e| format!("No se pudo listar la carpeta de sincronización: {e}"))?
        .filter_map(|entrada| entrada.ok().map(|e| e.path()))
        .filter(|ruta| ruta.is_file() && ruta.extension().is_some_and(|ext| ext == "ndjson"))
        .collect();

    archivos_ndjson.sort();

    let mut eventos_entrantes = Vec::new();
    let mut lineas_invalidas = Vec::new();

    for archivo in archivos_ndjson {
        let contenido = fs::read(&archivo)
            .map_err(|e| format!("Error al leer '{}': {e}", archivo.display()))?;
        let libro = serializador.deserializar_plano(&contenido)?;
        eventos_entrantes.extend(libro.eventos);
        lineas_invalidas.extend(libro.lineas_invalidas);
    }

    let mut resumen = repo_eventos.fusionar_lote_transaccional(&eventos_entrantes)?;
    resumen.total_recibidos += lineas_invalidas.len();
    resumen.rechazados.extend(lineas_invalidas);

    // Exportar el estado consolidado de este dispositivo en su archivo propio
    let device_id = repo_eventos.obtener_device_id()?;
    let eventos_locales = repo_eventos.listar_eventos_ordenados()?;
    let ahora = reloj.ahora();
    let bytes_salida = serializador.serializar_plano(&device_id, &ahora, &eventos_locales)?;

    let destino_propio = ruta_carpeta.join(format!("{device_id}.ndjson"));
    let ruta_tmp = ruta_temporal_atomica(&destino_propio);
    fs::write(&ruta_tmp, &bytes_salida)
        .map_err(|e| format!("No se pudo escribir el archivo de sincronización: {e}"))?;
    fs::rename(&ruta_tmp, &destino_propio).map_err(|e| {
        let _ = fs::remove_file(&ruta_tmp);
        format!("No se pudo actualizar el archivo del dispositivo: {e}")
    })?;

    Ok(resumen)
}