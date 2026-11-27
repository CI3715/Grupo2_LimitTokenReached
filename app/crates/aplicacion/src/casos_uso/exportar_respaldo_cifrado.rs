use std::fs;
use std::path::{Path, PathBuf};

use crate::puertos::{CifradorRespaldo, Reloj, RepositorioEventos, SerializadorLibro};

fn ruta_temporal_atomica(destino: &Path) -> PathBuf {
    let mut temp = destino.as_os_str().to_os_string();
    temp.push(".tmp");
    PathBuf::from(temp)
}

fn escribir_atomico(destino: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(padre) = destino.parent()
        && !padre.as_os_str().is_empty()
    {
        fs::create_dir_all(padre)
            .map_err(|e| format!("No se pudo crear la carpeta de destino: {e}"))?;
    }

    let ruta_tmp = ruta_temporal_atomica(destino);
    fs::write(&ruta_tmp, bytes)
        .map_err(|e| format!("No se pudo escribir el archivo temporal de respaldo: {e}"))?;
    fs::rename(&ruta_tmp, destino).map_err(|e| {
        let _ = fs::remove_file(&ruta_tmp);
        format!("No se pudo finalizar el guardado del respaldo: {e}")
    })
}

/// Exporta todo el libro filtrado, cifrado con Argon2id + AES-256-GCM y guardado atómicamente.
///
/// La capa de interfaz debe pedir la frase dos veces y advertir que, si la olvida,
/// el respaldo no se puede recuperar (§2.3 de la especificación).
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

    let bytes_planos = serializador.serializar_plano(&device_id, &reloj.ahora(), &eventos)?;
    let paquete_cifrado = cifrador.cifrar(&bytes_planos, frase_paso)?;
    escribir_atomico(ruta_destino, &paquete_cifrado)?;

    Ok(total_eventos)
}

/// Exporta el libro sin cifrar, con el mismo formato plano y filtrado de credenciales.
///
/// La interfaz debe advertir claramente que el archivo contiene información confidencial
/// y manejarlo como tal (RNF-04 / §2.3).
pub fn ejecutar_plano(
    ruta_destino: &Path,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    reloj: &dyn Reloj,
) -> Result<usize, String> {
    let device_id = repo_eventos.obtener_device_id()?;
    let eventos = repo_eventos.listar_eventos_ordenados()?;
    let total_eventos = eventos.len();

    let bytes_planos = serializador.serializar_plano(&device_id, &reloj.ahora(), &eventos)?;
    escribir_atomico(ruta_destino, &bytes_planos)?;

    Ok(total_eventos)
}
