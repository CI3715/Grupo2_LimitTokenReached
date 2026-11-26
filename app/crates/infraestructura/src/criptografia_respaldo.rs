use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Payload},
};
use aplicacion::puertos::CifradorRespaldo;
use argon2::{Algorithm, Argon2, Params, Version};
use rand::{RngCore, rngs::OsRng};
use zeroize::Zeroizing;

/// Identificador mágico y versión del contenedor binario cifrado (.ccbak).
/// No contiene ningún dato del libro personal (RF-INT-06).
const MAGIC_RESPALDO: &[u8; 6] = b"CCBAK\x01";
const LONGITUD_SALT: usize = 16;
const LONGITUD_NONCE: usize = 12;
const LONGITUD_CABECERA: usize = 6 + 4 + 4 + 4 + LONGITUD_SALT + LONGITUD_NONCE; // 46 bytes
const LONGITUD_MINIMA_FRASE: usize = 12;

// Parámetros Argon2id (64 MiB de memoria, 3 iteraciones, 4 carriles)
const ARGON2_MEMORIA_KIB: u32 = 65_536;
const ARGON2_ITERACIONES: u32 = 3;
const ARGON2_PARALELISMO: u32 = 4;

const ERROR_DESCIFRADO: &str = "Frase de paso incorrecta o archivo dañado";

fn derivar_clave_argon2id(
    frase_paso: &str,
    salt: &[u8],
    memoria_kib: u32,
    iteraciones: u32,
    paralelismo: u32,
) -> Result<Zeroizing<[u8; 32]>, String> {
    let params = Params::new(memoria_kib, iteraciones, paralelismo, Some(32))
        .map_err(|_| ERROR_DESCIFRADO.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut clave = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(frase_paso.as_bytes(), salt, &mut *clave)
        .map_err(|_| ERROR_DESCIFRADO.to_string())?;

    Ok(clave)
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

        // 1. Generar sal y nonce criptográficamente aleatorios nuevos en cada exportación
        let mut salt = [0u8; LONGITUD_SALT];
        let mut nonce_bytes = [0u8; LONGITUD_NONCE];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        // 2. Derivar clave de 256 bits con Argon2id (limpiada automáticamente con Zeroizing)
        let clave = derivar_clave_argon2id(
            frase_paso,
            &salt,
            ARGON2_MEMORIA_KIB,
            ARGON2_ITERACIONES,
            ARGON2_PARALELISMO,
        )?;

        // 3. Construir cabecera en claro (sin datos del libro) que actuará como AAD
        let mut cabecera = Vec::with_capacity(LONGITUD_CABECERA);
        cabecera.extend_from_slice(MAGIC_RESPALDO);
        cabecera.extend_from_slice(&ARGON2_MEMORIA_KIB.to_be_bytes());
        cabecera.extend_from_slice(&ARGON2_ITERACIONES.to_be_bytes());
        cabecera.extend_from_slice(&ARGON2_PARALELISMO.to_be_bytes());
        cabecera.extend_from_slice(&salt);
        cabecera.extend_from_slice(&nonce_bytes);

        // 4. Cifrado autenticado AES-256-GCM vinculando la cabecera como AAD
        let cipher = Aes256Gcm::new_from_slice(&*clave)
            .map_err(|_| "No se pudo inicializar el cifrador AES-256-GCM.".to_string())?;
        let nonce = Nonce::from(nonce_bytes);
        let texto_cifrado = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: datos_planos,
                    aad: &cabecera,
                },
            )
            .map_err(|_| "Error al cifrar el respaldo.".to_string())?;

        let mut paquete = cabecera;
        paquete.extend_from_slice(&texto_cifrado);
        Ok(paquete)
    }

    fn descifrar(&self, paquete_cifrado: &[u8], frase_paso: &str) -> Result<Vec<u8>, String> {
        if frase_paso.is_empty() || paquete_cifrado.len() <= LONGITUD_CABECERA {
            return Err(ERROR_DESCIFRADO.to_string());
        }

        // 1. Verificar identificador mágico y versión
        if &paquete_cifrado[0..6] != MAGIC_RESPALDO {
            return Err(ERROR_DESCIFRADO.to_string());
        }

        // 2. Extraer parámetros de derivación, sal, nonce y texto cifrado
        let memoria_kib = u32::from_be_bytes(
            paquete_cifrado[6..10]
                .try_into()
                .map_err(|_| ERROR_DESCIFRADO.to_string())?,
        );
        let iteraciones = u32::from_be_bytes(
            paquete_cifrado[10..14]
                .try_into()
                .map_err(|_| ERROR_DESCIFRADO.to_string())?,
        );
        let paralelismo = u32::from_be_bytes(
            paquete_cifrado[14..18]
                .try_into()
                .map_err(|_| ERROR_DESCIFRADO.to_string())?,
        );
        let salt = &paquete_cifrado[18..34];
        let nonce_bytes: [u8; LONGITUD_NONCE] = paquete_cifrado[34..46]
            .try_into()
            .map_err(|_| ERROR_DESCIFRADO.to_string())?;

        let cabecera_aad = &paquete_cifrado[0..LONGITUD_CABECERA];
        let ciphertext = &paquete_cifrado[LONGITUD_CABECERA..];

        // 3. Re-derivar clave con Argon2id y verificar etiqueta de autenticación AES-256-GCM
        let clave =
            derivar_clave_argon2id(frase_paso, salt, memoria_kib, iteraciones, paralelismo)?;
        let cipher =
            Aes256Gcm::new_from_slice(&*clave).map_err(|_| ERROR_DESCIFRADO.to_string())?;
        let nonce = Nonce::from(nonce_bytes);

        cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: ciphertext,
                    aad: cabecera_aad,
                },
            )
            .map_err(|_| ERROR_DESCIFRADO.to_string())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn cifra_y_descifra_con_sal_y_nonce_unicos_por_exportacion() {
        let cifrador = CifradorRespaldoAesGcm::new();
        let plano = b"contenido secreto del libro personal";
        let frase = "frase-de-paso-segura-2026";

        let exp1 = cifrador.cifrar(plano, frase).unwrap();
        let exp2 = cifrador.cifrar(plano, frase).unwrap();

        // Cada exportación usa sal y nonce distintos
        assert_ne!(&exp1[18..34], &exp2[18..34]);
        assert_ne!(&exp1[34..46], &exp2[34..46]);
        assert_ne!(exp1, exp2);

        let descifrado1 = cifrador.descifrar(&exp1, frase).unwrap();
        let descifrado2 = cifrador.descifrar(&exp2, frase).unwrap();
        assert_eq!(descifrado1, plano);
        assert_eq!(descifrado2, plano);
    }

    #[test]
    fn rechaza_frase_corta_frase_incorrecta_o_archivo_alterado() {
        let cifrador = CifradorRespaldoAesGcm::new();
        assert!(cifrador.cifrar(b"datos", "corta").is_err());

        let mut paquete = cifrador
            .cifrar(b"datos del libro", "frase-valida-1234")
            .unwrap();

        assert_eq!(
            cifrador.descifrar(&paquete, "frase-equivocada-999").unwrap_err(),
            ERROR_DESCIFRADO
        );

        // Alterar 1 bit del ciphertext
        let ultimo = paquete.len() - 1;
        paquete[ultimo] ^= 0x01;
        assert_eq!(
            cifrador.descifrar(&paquete, "frase-valida-1234").unwrap_err(),
            ERROR_DESCIFRADO
        );
    }
}