//! Cifrado autenticado del respaldo con frase de paso (Entrega 8).
//!
//! * KDF: Argon2id (m=64 MiB, t=3, p=4), sal aleatoria de 32 bytes por exportación.
//! * AEAD: AES-256-GCM, nonce aleatorio de 12 bytes y etiqueta de 16 bytes.
//! * La cabecera en claro solo contiene formato, versión y parámetros públicos;
//!   se autentica completa como AAD para detectar cualquier modificación.

use std::collections::HashMap;

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Payload},
};
use aplicacion::puertos::{CifradorRespaldo, MAGIA_RESPALDO_CIFRADO};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::{RngCore, rngs::OsRng};
use zeroize::Zeroizing;

pub const VERSION_FORMATO: &str = "1.0";

const FORMATO: &str = "1";
const ALGORITMO: &str = "AES-256-GCM";
const KDF: &str = "ARGON2ID";
const PARAMETROS: &str = "M=65536,T=3,P=4";
const LONGITUD_SAL: usize = 32;
const LONGITUD_NONCE: usize = 12;
const LONGITUD_CLAVE: usize = 32;
const LONGITUD_MINIMA_FRASE: usize = 12;

const ERROR_DESCIFRADO: &str = "Frase de paso incorrecta o archivo dañado.";

fn error_version(version: &str) -> String {
    format!(
        "Versión de formato no soportada: {version}. La aplicación soporta la versión {VERSION_FORMATO}."
    )
}

fn derivar_clave_argon2id(
    frase_paso: &str,
    salt: &[u8],
) -> Result<Zeroizing<[u8; LONGITUD_CLAVE]>, String> {
    let params = Params::new(65_536, 3, 4, Some(LONGITUD_CLAVE))
        .map_err(|_| ERROR_DESCIFRADO.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut clave = Zeroizing::new([0u8; LONGITUD_CLAVE]);
    argon2
        .hash_password_into(frase_paso.as_bytes(), salt, &mut *clave)
        .map_err(|_| ERROR_DESCIFRADO.to_string())?;

    Ok(clave)
}

fn leer_campos(cabecera: &str) -> Result<HashMap<&str, &str>, String> {
    let mut campos = HashMap::new();
    for linea in cabecera.lines().skip(1) {
        let (clave, valor) = linea
            .split_once('=')
            .ok_or_else(|| ERROR_DESCIFRADO.to_string())?;
        campos.insert(clave.trim(), valor.trim());
    }
    Ok(campos)
}

pub struct CifradorRespaldoAesGcm;

impl CifradorRespaldoAesGcm {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CifradorRespaldoAesGcm {
    fn default() -> Self {
        Self::new()
    }
}

impl CifradorRespaldo for CifradorRespaldoAesGcm {
    fn cifrar(&self, datos_planos: &[u8], frase_paso: &str) -> Result<Vec<u8>, String> {
        if frase_paso.chars().count() < LONGITUD_MINIMA_FRASE {
            return Err(format!(
                "La frase de paso debe tener al menos {LONGITUD_MINIMA_FRASE} caracteres."
            ));
        }

        let mut salt = [0u8; LONGITUD_SAL];
        let mut nonce_bytes = [0u8; LONGITUD_NONCE];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        let clave = derivar_clave_argon2id(frase_paso, &salt)?;
        let cabecera = format!(
            "{MAGIA_RESPALDO_CIFRADO}\nFORMATO={FORMATO}\nVERSION={VERSION_FORMATO}\nCIFRADO={ALGORITMO}\nKDF={KDF}\nPARAMETROS={PARAMETROS}\nSALT={}\nNONCE={}\n",
            STANDARD.encode(salt),
            STANDARD.encode(nonce_bytes)
        );

        let cifrador = Aes256Gcm::new_from_slice(&*clave)
            .map_err(|_| "No se pudo inicializar el cifrador AES-256-GCM.".to_string())?;
        let texto_cifrado = cifrador
            .encrypt(
                &Nonce::from(nonce_bytes),
                Payload {
                    msg: datos_planos,
                    aad: cabecera.as_bytes(),
                },
            )
            .map_err(|_| "Error al cifrar el respaldo.".to_string())?;

        Ok(format!("{cabecera}DATOS={}\n", STANDARD.encode(texto_cifrado)).into_bytes())
    }

    fn descifrar(&self, paquete_cifrado: &[u8], frase_paso: &str) -> Result<Vec<u8>, String> {
        if frase_paso.is_empty() {
            return Err(ERROR_DESCIFRADO.to_string());
        }
        let archivo =
            std::str::from_utf8(paquete_cifrado).map_err(|_| ERROR_DESCIFRADO.to_string())?;
        let inicio_datos = archivo
            .find("\nDATOS=")
            .ok_or_else(|| ERROR_DESCIFRADO.to_string())?;
        let cabecera = &archivo[..inicio_datos + 1];
        let datos = archivo[inicio_datos + "\nDATOS=".len()..].trim();

        let mut lineas = cabecera.lines();
        if lineas.next() != Some(MAGIA_RESPALDO_CIFRADO) {
            return Err(ERROR_DESCIFRADO.to_string());
        }
        let campos = leer_campos(cabecera)?;

        match campos.get("VERSION").copied() {
            Some(version) if version == VERSION_FORMATO => {}
            Some(version) => return Err(error_version(version)),
            None => return Err(ERROR_DESCIFRADO.to_string()),
        }
        if campos.get("FORMATO").copied() != Some(FORMATO)
            || campos.get("CIFRADO").copied() != Some(ALGORITMO)
            || campos.get("KDF").copied() != Some(KDF)
            || campos.get("PARAMETROS").copied() != Some(PARAMETROS)
        {
            return Err(ERROR_DESCIFRADO.to_string());
        }

        let salt = STANDARD
            .decode(
                campos
                    .get("SALT")
                    .ok_or_else(|| ERROR_DESCIFRADO.to_string())?,
            )
            .map_err(|_| ERROR_DESCIFRADO.to_string())?;
        let nonce = STANDARD
            .decode(
                campos
                    .get("NONCE")
                    .ok_or_else(|| ERROR_DESCIFRADO.to_string())?,
            )
            .map_err(|_| ERROR_DESCIFRADO.to_string())?;
        let contenido = STANDARD
            .decode(datos)
            .map_err(|_| ERROR_DESCIFRADO.to_string())?;
        if salt.len() != LONGITUD_SAL || nonce.len() != LONGITUD_NONCE {
            return Err(ERROR_DESCIFRADO.to_string());
        }

        let clave = derivar_clave_argon2id(frase_paso, &salt)?;
        let cifrador =
            Aes256Gcm::new_from_slice(&*clave).map_err(|_| ERROR_DESCIFRADO.to_string())?;
        let nonce: [u8; LONGITUD_NONCE] =
            nonce.try_into().map_err(|_| ERROR_DESCIFRADO.to_string())?;

        cifrador
            .decrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: &contenido,
                    aad: cabecera.as_bytes(),
                },
            )
            .map_err(|_| ERROR_DESCIFRADO.to_string())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn valor_de(paquete: &str, clave: &str) -> String {
        paquete
            .lines()
            .find_map(|linea| linea.strip_prefix(&format!("{clave}=")))
            .unwrap()
            .to_string()
    }

    #[test]
    fn cifra_y_descifra_con_sal_y_nonce_unicos_por_exportacion() {
        let cifrador = CifradorRespaldoAesGcm::new();
        let plano = b"contenido secreto del libro personal";
        let frase = "frase-de-paso-segura-2026";

        let exp1 = String::from_utf8(cifrador.cifrar(plano, frase).unwrap()).unwrap();
        let exp2 = String::from_utf8(cifrador.cifrar(plano, frase).unwrap()).unwrap();

        assert_ne!(valor_de(&exp1, "SALT"), valor_de(&exp2, "SALT"));
        assert_ne!(valor_de(&exp1, "NONCE"), valor_de(&exp2, "NONCE"));
        assert_ne!(exp1, exp2);
        assert!(exp1.starts_with(MAGIA_RESPALDO_CIFRADO));

        assert_eq!(cifrador.descifrar(exp1.as_bytes(), frase).unwrap(), plano);
        assert_eq!(cifrador.descifrar(exp2.as_bytes(), frase).unwrap(), plano);
    }

    #[test]
    fn rechaza_frase_corta_frase_incorrecta_archivo_alterado_y_version() {
        let cifrador = CifradorRespaldoAesGcm::new();
        assert!(cifrador.cifrar(b"datos", "corta").is_err());

        let paquete = String::from_utf8(
            cifrador
                .cifrar(b"datos del libro", "frase-valida-1234")
                .unwrap(),
        )
        .unwrap();

        assert_eq!(
            cifrador
                .descifrar(paquete.as_bytes(), "frase-equivocada-999")
                .unwrap_err(),
            ERROR_DESCIFRADO
        );

        let mut alterado = paquete.clone();
        let datos = valor_de(&paquete, "DATOS");
        alterado = alterado.replace(&format!("DATOS={datos}"), "DATOS=YmFk");
        assert_eq!(
            cifrador
                .descifrar(alterado.as_bytes(), "frase-valida-1234")
                .unwrap_err(),
            ERROR_DESCIFRADO
        );

        let otra_version = paquete.replace("VERSION=1.0", "VERSION=9.9.9");
        let error = cifrador
            .descifrar(otra_version.as_bytes(), "frase-valida-1234")
            .unwrap_err();
        assert!(error.contains("Versión de formato no soportada: 9.9.9"));
        assert!(error.contains(VERSION_FORMATO));
    }
}
