use std::fs;
use std::path::{Path, PathBuf};

use crate::puertos::{CifradorRespaldo, Reloj, RepositorioEventos, SerializadorLibro};

fn ruta_temporal_atomica(destino: &Path) -> PathBuf {
    let mut temp = destino.as_os_str().to_os_string();
    temp.push(".tmp");
    PathBuf::from(temp)
}

/// Exporta todo el libro de eventos filtrado (sin tokens ni credenciales),
/// cifrado con Argon2id + AES-256-GCM y guardado de forma atómica en disco.
pub fn ejecutar(
    ruta_destino: &Path,
    frase_paso: &str,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
    reloj: &dyn Reloj,
) -> Result<usize, String> {
    let device_id = repo_eventos.obtener_device_id()?;
    let eventos = repo_eventos.listar_eventos_ordenados()?;
    let total_eventos = eventos.len();
    let ahora = reloj.ahora();

    let bytes_planos = serializador.serializar_plano(&device_id, &ahora, &eventos)?;
    let paquete_cifrado = cifrador.cifrar(&bytes_planos, frase_paso)?;

    if let Some(padre) = ruta_destino.parent()
        && !padre.as_os_str().is_empty()
    {
        fs::create_dir_all(padre)
            .map_err(|e| format!("No se pudo crear la carpeta de destino: {e}"))?;
    }

    let ruta_tmp = ruta_temporal_atomica(ruta_destino);
    fs::write(&ruta_tmp, &paquete_cifrado)
        .map_err(|e| format!("No se pudo escribir el archivo temporal de respaldo: {e}"))?;
    fs::rename(&ruta_tmp, ruta_destino).map_err(|e| {
        let _ = fs::remove_file(&ruta_tmp);
        format!("No se pudo finalizar el guardado del respaldo: {e}")
    })?;

    Ok(total_eventos)
}