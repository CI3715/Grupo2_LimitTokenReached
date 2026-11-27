//! Implementación del puerto `AlmacenSecretos` con el almacén del sistema operativo.

use aplicacion::puertos::AlmacenSecretos;
use keyring::{Entry, Error};

const SERVICIO: &str = "cuentas-claras";

/// Guarda los secretos de la sincronización en el keychain de Windows,
/// el llavero de macOS o el secret service de Linux.
pub struct AlmacenSecretosSistema;

impl AlmacenSecretosSistema {
    fn entrada(clave: &str) -> Result<Entry, String> {
        Entry::new(SERVICIO, clave)
            .map_err(|_| "No se pudo acceder al almacén de secretos.".to_string())
    }
}

impl AlmacenSecretos for AlmacenSecretosSistema {
    fn guardar(&self, clave: &str, valor: &str) -> Result<(), String> {
        Self::entrada(clave)?
            .set_password(valor)
            .map_err(|_| "No se pudo guardar en el almacén de secretos.".to_string())
    }

    fn leer(&self, clave: &str) -> Result<Option<String>, String> {
        match Self::entrada(clave)?.get_password() {
            Ok(valor) => Ok(Some(valor)),
            Err(Error::NoEntry) => Ok(None),
            Err(_) => Err("No se pudo leer el almacén de secretos.".to_string()),
        }
    }

    fn borrar(&self, clave: &str) -> Result<(), String> {
        match Self::entrada(clave)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(Error::NoEntry) => Ok(()),
            Err(_) => Err("No se pudo borrar del almacén de secretos.".to_string()),
        }
    }
}
