//! Configuración del destino de sincronización desde la aplicación.
//!
//! La información no secreta y los secretos viven en el mismo almacén del
//! sistema operativo bajo claves estables. El destino y la exportación nunca
//! reciben ni la frase de paso ni las credenciales de Git.

use crate::puertos::AlmacenSecretos;
use contrato::ConfigDestinoSyncDTO;

pub const CLAVE_TIPO: &str = "sync_tipo";
pub const CLAVE_RUTA: &str = "sync_ruta_o_url";
pub const CLAVE_CREDENCIALES: &str = "sync_credenciales_git";
pub const CLAVE_FRASE: &str = "sync_frase_de_paso";

const LONGITUD_MINIMA_FRASE: usize = 12;

/// Guarda el destino elegido. Las credenciales y la frase se escriben solo en
/// el almacén de secretos; si no hay credenciales, se borra la entrada anterior.
pub fn guardar(
    configuracion: ConfigDestinoSyncDTO,
    almacen: &dyn AlmacenSecretos,
) -> Result<(), String> {
    let tipo = configuracion.tipo.trim();
    if tipo != "git" && tipo != "carpeta" {
        return Err("El tipo de destino debe ser \"git\" o \"carpeta\".".to_string());
    }

    let ruta = configuracion.ruta_o_url.trim();
    if ruta.is_empty() {
        return Err("La ruta o URL del destino no puede estar vacía.".to_string());
    }

    let frase = configuracion
        .frase_de_paso
        .as_deref()
        .map(str::trim)
        .filter(|valor| !valor.is_empty())
        .ok_or_else(|| "La frase de paso es obligatoria para sincronizar.".to_string())?;
    if frase.len() < LONGITUD_MINIMA_FRASE {
        return Err("La frase de paso debe tener al menos 12 caracteres.".to_string());
    }

    let credenciales = configuracion
        .credenciales
        .as_deref()
        .map(str::trim)
        .filter(|valor| !valor.is_empty());
    if tipo == "carpeta" && credenciales.is_some() {
        return Err("La carpeta compartida no usa credenciales de Git.".to_string());
    }

    almacen.guardar(CLAVE_TIPO, tipo)?;
    almacen.guardar(CLAVE_RUTA, ruta)?;
    almacen.guardar(CLAVE_FRASE, frase)?;
    match credenciales {
        Some(valor) => almacen.guardar(CLAVE_CREDENCIALES, valor)?,
        None => almacen.borrar(CLAVE_CREDENCIALES)?,
    }

    Ok(())
}

/// Devuelve la configuración completa para uso interno. Incluye los secretos:
/// los comandos que responden al frontend deben quitarlos antes de serializar.
pub fn leer(almacen: &dyn AlmacenSecretos) -> Result<Option<ConfigDestinoSyncDTO>, String> {
    let Some(tipo) = almacen.leer(CLAVE_TIPO)? else {
        return Ok(None);
    };
    let ruta_o_url = almacen
        .leer(CLAVE_RUTA)?
        .ok_or_else(|| "La configuración del destino está incompleta.".to_string())?;
    let frase_de_paso = almacen
        .leer(CLAVE_FRASE)?
        .ok_or_else(|| "La frase de paso guardada no existe.".to_string())?;
    let credenciales = almacen.leer(CLAVE_CREDENCIALES)?;

    Ok(Some(ConfigDestinoSyncDTO {
        tipo,
        ruta_o_url,
        credenciales,
        frase_de_paso: Some(frase_de_paso),
    }))
}

/// Elimina el destino configurado y todos sus secretos.
pub fn borrar(almacen: &dyn AlmacenSecretos) -> Result<(), String> {
    for clave in [CLAVE_TIPO, CLAVE_RUTA, CLAVE_CREDENCIALES, CLAVE_FRASE] {
        almacen.borrar(clave)?;
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::{collections::HashMap, sync::Mutex};

    #[derive(Default)]
    struct AlmacenFalso {
        valores: Mutex<HashMap<String, String>>,
    }

    impl AlmacenSecretos for AlmacenFalso {
        fn guardar(&self, clave: &str, valor: &str) -> Result<(), String> {
            self.valores
                .lock()
                .unwrap()
                .insert(clave.to_string(), valor.to_string());
            Ok(())
        }

        fn leer(&self, clave: &str) -> Result<Option<String>, String> {
            Ok(self.valores.lock().unwrap().get(clave).cloned())
        }

        fn borrar(&self, clave: &str) -> Result<(), String> {
            self.valores.lock().unwrap().remove(clave);
            Ok(())
        }
    }

    fn configuracion(tipo: &str) -> ConfigDestinoSyncDTO {
        ConfigDestinoSyncDTO {
            tipo: tipo.to_string(),
            ruta_o_url: "https://github.com/usuario/libro.git".to_string(),
            credenciales: Some("ghp_token".to_string()),
            frase_de_paso: Some("frase-segura-123".to_string()),
        }
    }

    #[test]
    fn guarda_y_devuelve_la_configuracion_con_sus_secretos() {
        let almacen = AlmacenFalso::default();

        guardar(configuracion("git"), &almacen).unwrap();
        let leida = leer(&almacen).unwrap().unwrap();

        assert_eq!(leida.tipo, "git");
        assert_eq!(leida.ruta_o_url, "https://github.com/usuario/libro.git");
        assert_eq!(leida.credenciales.as_deref(), Some("ghp_token"));
        assert_eq!(leida.frase_de_paso.as_deref(), Some("frase-segura-123"));

        let valores = almacen.valores.lock().unwrap();
        assert_eq!(valores.get(CLAVE_CREDENCIALES).unwrap(), "ghp_token");
        assert_eq!(valores.get(CLAVE_FRASE).unwrap(), "frase-segura-123");
        assert!(!valores.get(CLAVE_RUTA).unwrap().contains("ghp_token"));
    }

    #[test]
    fn la_carpeta_no_guarda_credenciales_y_borrar_elimina_todo() {
        let almacen = AlmacenFalso::default();
        guardar(configuracion("git"), &almacen).unwrap();

        let mut datos = configuracion("carpeta");
        datos.credenciales = None;
        guardar(datos, &almacen).unwrap();

        assert!(
            almacen
                .valores
                .lock()
                .unwrap()
                .get(CLAVE_CREDENCIALES)
                .is_none()
        );
        assert_eq!(leer(&almacen).unwrap().unwrap().tipo, "carpeta");

        borrar(&almacen).unwrap();
        assert!(leer(&almacen).unwrap().is_none());
    }

    #[test]
    fn rechaza_datos_invalidos_sin_sobrescriber_la_configuracion() {
        let almacen = AlmacenFalso::default();
        guardar(configuracion("git"), &almacen).unwrap();
        let antes = almacen.valores.lock().unwrap().clone();

        let mut corta = configuracion("git");
        corta.frase_de_paso = Some("corta".to_string());
        assert_eq!(
            guardar(corta, &almacen).unwrap_err(),
            "La frase de paso debe tener al menos 12 caracteres."
        );

        let mut carpeta = configuracion("carpeta");
        assert_eq!(
            guardar(carpeta.clone(), &almacen).unwrap_err(),
            "La carpeta compartida no usa credenciales de Git."
        );

        carpeta.ruta_o_url.clear();
        assert_eq!(
            guardar(carpeta, &almacen).unwrap_err(),
            "La ruta o URL del destino no puede estar vacía."
        );

        assert_eq!(*almacen.valores.lock().unwrap(), antes);
    }
}
