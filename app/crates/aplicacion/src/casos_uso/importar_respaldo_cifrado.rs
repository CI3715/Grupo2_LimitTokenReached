use std::fs;
use std::path::Path;

use dominio::libro::ResumenFusion;

use crate::puertos::{
    CifradorRespaldo, MAGIA_RESPALDO_CIFRADO, RepositorioEventos, SerializadorLibro,
};

/// Descifra (si hace falta) y valida el archivo completo antes de tocar la base.
///
/// Un archivo cifrado se reconoce por su cabecera en claro; si no la tiene se trata
/// como exportación sin cifrar. En ambos casos se valida versión, encabezado y
/// sintaxis antes de abrir la transacción de fusión (§2.4 de la especificación).
pub fn ejecutar(
    ruta_origen: &Path,
    frase_paso: &str,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
) -> Result<ResumenFusion, String> {
    let paquete = fs::read(ruta_origen)
        .map_err(|e| format!("No se pudo leer el archivo de respaldo: {e}"))?;

    let bytes_planos = if paquete.starts_with(MAGIA_RESPALDO_CIFRADO.as_bytes()) {
        cifrador.descifrar(&paquete, frase_paso)?
    } else {
        paquete
    };

    let libro = serializador.deserializar_plano(&bytes_planos)?;
    repo_eventos.fusionar_lote_transaccional(&libro.eventos)
}
