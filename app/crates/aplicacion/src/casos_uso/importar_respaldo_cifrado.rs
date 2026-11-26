use std::fs;
use std::path::Path;

use dominio::libro::ResumenFusion;

use crate::puertos::{CifradorRespaldo, RepositorioEventos, SerializadorLibro};

/// Descifra un archivo de respaldo, valida sus líneas y fusiona sus eventos
/// de forma idempotente y transaccional en el libro local.
pub fn ejecutar(
    ruta_origen: &Path,
    frase_paso: &str,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
) -> Result<ResumenFusion, String> {
    let paquete_cifrado = fs::read(ruta_origen)
        .map_err(|e| format!("No se pudo leer el archivo de respaldo: {e}"))?;

    let bytes_planos = cifrador.descifrar(&paquete_cifrado, frase_paso)?;
    let libro = serializador.deserializar_plano(&bytes_planos)?;

    let mut resumen = repo_eventos.fusionar_lote_transaccional(&libro.eventos)?;
    resumen.total_recibidos += libro.lineas_invalidas.len();
    resumen.rechazados.extend(libro.lineas_invalidas);

    Ok(resumen)
}