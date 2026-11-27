use std::sync::Mutex;
use aplicacion::puertos::{LoteRecibido, TransporteSincronizacion};

/// Adaptador falso en memoria para pruebas de contrato 
/// Simula un destino de sincronización sin tocar el sistema de archivos ni Git
pub struct AdaptadorMemoria {
    lotes: Mutex<Vec<LoteRecibido>>,
}

impl AdaptadorMemoria {
    pub fn nuevo() -> Self {
        Self {
            lotes: Mutex::new(Vec::new()),
        }
    }
}

impl TransporteSincronizacion for AdaptadorMemoria {
    fn leer_lotes(&self) -> Result<Vec<LoteRecibido>, String> {
        let lotes = self.lotes.lock().map_err(|e| e.to_string())?;
        Ok(lotes.clone())
    }

    fn publicar_lote(&self, dispositivo: &str, contenido: &[u8]) -> Result<(), String> {
        let mut lotes = self.lotes.lock().map_err(|e| e.to_string())?;
        lotes.push(LoteRecibido {
            dispositivo: dispositivo.to_string(),
            contenido: contenido.to_vec(),
        });
        Ok(())
    }
}