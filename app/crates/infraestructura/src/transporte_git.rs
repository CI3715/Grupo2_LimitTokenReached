//! Adaptador de `TransporteSincronizacion` basado en el CLI de Git.
//!
//! Los lotes cifrados viven en la carpeta del dispositivo dentro del repositorio.
//! Las credenciales llegan desde el almacén de secretos del sistema operativo y
//! se inyectan solo en cada proceso `git`: nunca se escriben en el libro, en el
//! destino ni en la exportación.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use aplicacion::puertos::{LoteRecibido, TransporteSincronizacion};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use uuid::Uuid;

const ERROR_REPOSITORIO: &str = "No se pudo llegar al repositorio; se reintentará.";
const ERROR_CREDENCIALES: &str = "Las credenciales fueron rechazadas; cómo actualizarlas.";

/// Sincroniza los lotes del libro con un repositorio Git propio del usuario.
#[derive(Debug, Clone)]
pub struct TransporteGit {
    url: String,
    directorio: PathBuf,
    credenciales: Option<String>,
}

impl TransporteGit {
    pub fn new(url: impl Into<String>, directorio: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            directorio: directorio.into(),
            credenciales: None,
        }
    }

    /// Configura el token leído del almacén de secretos para esta sincronización.
    /// No se guarda en el repositorio ni persiste en el adaptador.
    pub fn con_credenciales(mut self, credenciales: Option<String>) -> Self {
        self.credenciales = credenciales;
        self
    }

    fn cabecera(&self) -> Option<String> {
        let credenciales = self.credenciales.as_deref()?;
        if !es_http(&self.url) {
            return None;
        }
        Some(cabecera_autorizacion(credenciales))
    }

    /// Clona el repositorio si todavía no existe un clon local.
    /// Expuesto para poder configurar la identidad de Git antes de sincronizar en las pruebas.
    pub fn asegurar_clon(&self) -> Result<(), String> {
        if self.directorio.join(".git").exists() {
            return Ok(());
        }

        if self.directorio.exists() {
            let vacio = fs::read_dir(&self.directorio)
                .map_err(|_| ERROR_REPOSITORIO.to_string())?
                .next()
                .is_none();
            if !vacio {
                return Err(ERROR_REPOSITORIO.to_string());
            }
        } else if let Some(padre) = self.directorio.parent() {
            fs::create_dir_all(padre).map_err(|_| ERROR_REPOSITORIO.to_string())?;
        }

        let destino = self.directorio.to_string_lossy().into_owned();
        ejecutar_git(
            None,
            &["clone", "--quiet", self.url.as_str(), destino.as_str()],
            self.cabecera().as_deref(),
        )?;
        Ok(())
    }

    fn git(&self, argumentos: &[&str]) -> Result<String, String> {
        ejecutar_git(
            Some(&self.directorio),
            argumentos,
            self.cabecera().as_deref(),
        )
    }

    fn rama_actual(&self) -> Result<String, String> {
        // `symbolic-ref` funciona también en un clon sin commits, donde
        // `rev-parse HEAD` falla porque todavía no existe la revisión.
        Ok(self
            .git(&["symbolic-ref", "--quiet", "--short", "HEAD"])?
            .trim()
            .to_string())
    }
}

impl TransporteSincronizacion for TransporteGit {
    fn leer_lotes(&self) -> Result<Vec<LoteRecibido>, String> {
        self.asegurar_clon()?;
        self.git(&["fetch", "--quiet", "origin"])?;

        let rama = self.rama_actual()?;
        let referencia_remota = format!("refs/remotes/origin/{rama}");

        if self
            .git(&["rev-parse", "--verify", "--quiet", "HEAD"])
            .is_ok()
        {
            self.git(&["pull", "--quiet", "--no-edit", "--rebase", "origin", &rama])?;
        } else if self
            .git(&["show-ref", "--verify", "--quiet", &referencia_remota])
            .is_ok()
        {
            // El clon está vacío pero el remoto ya tiene commits: adoptamos su rama.
            self.git(&["reset", "--hard", &format!("origin/{rama}")])?;
        }
        // Si ambos están vacíos todavía no hay nada que traer.

        recoger_lotes(&self.directorio)
    }

    fn publicar_lote(&self, dispositivo: &str, contenido: &[u8]) -> Result<(), String> {
        self.asegurar_clon()?;

        let nombre = Path::new(dispositivo)
            .file_name()
            .and_then(|valor| valor.to_str())
            .filter(|valor| !valor.is_empty())
            .unwrap_or("desconocido");
        let carpeta = self.directorio.join(nombre);
        fs::create_dir_all(&carpeta).map_err(|_| ERROR_REPOSITORIO.to_string())?;

        let ruta = carpeta.join(format!("lote-{}.ccbak", Uuid::new_v4()));
        let temporal = ruta.with_extension("tmp");
        // Git puede convertir saltos de línea en archivos de texto. El paquete cifrado
        // autentica su cabecera byte a byte, así que se guarda en base64 sin saltos.
        let contenido_git = STANDARD.encode(contenido);
        fs::write(&temporal, contenido_git).map_err(|_| ERROR_REPOSITORIO.to_string())?;
        fs::rename(&temporal, &ruta).map_err(|_| {
            let _ = fs::remove_file(&temporal);
            ERROR_REPOSITORIO.to_string()
        })?;

        let relativo = ruta
            .strip_prefix(&self.directorio)
            .unwrap_or(&ruta)
            .to_string_lossy()
            .replace('\\', "/");
        self.git(&["add", "--", &relativo])?;
        self.git(&[
            "commit",
            "--quiet",
            "-m",
            "Lote de sincronizacion del libro",
        ])?;

        let rama = self.rama_actual()?;
        if let Err(error) = self.git(&["push", "--quiet", "origin", &rama]) {
            if error == ERROR_CREDENCIALES {
                return Err(error);
            }

            // El remoto cambió mientras se publicaba: se trae y se reintenta una vez.
            self.git(&["fetch", "--quiet", "origin"])?;
            self.git(&["pull", "--quiet", "--no-edit", "--rebase", "origin", &rama])?;
            self.git(&["push", "--quiet", "origin", &rama])?;
        }

        Ok(())
    }
}

fn ejecutar_git(
    directorio: Option<&Path>,
    argumentos: &[&str],
    cabecera: Option<&str>,
) -> Result<String, String> {
    let mut comando = Command::new("git");
    if let Some(directorio) = directorio {
        comando.arg("-C").arg(directorio);
    }
    if let Some(cabecera) = cabecera {
        // GIT_CONFIG_* evita que el token aparezca en la lista de argumentos.
        comando
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "http.extraheader")
            .env("GIT_CONFIG_VALUE_0", cabecera);
    }

    let salida = comando
        .args(argumentos)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|_| ERROR_REPOSITORIO.to_string())?;

    if salida.status.success() {
        Ok(String::from_utf8_lossy(&salida.stdout).into_owned())
    } else {
        Err(clasificar_error(&String::from_utf8_lossy(&salida.stderr)))
    }
}

fn es_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// GitHub acepta `x-access-token:<token>`; si el usuario guardó `usuario:token`,
/// se respeta tal cual. El valor resultante viaja solo dentro del proceso Git.
fn cabecera_autorizacion(credenciales: &str) -> String {
    let usuario_token = match credenciales.split_once(':') {
        Some((usuario, token)) if !usuario.is_empty() && !token.is_empty() => {
            credenciales.to_string()
        }
        _ => format!("x-access-token:{credenciales}"),
    };
    format!("Authorization: Basic {}", STANDARD.encode(usuario_token))
}

fn clasificar_error(stderr: &str) -> String {
    let mensaje = stderr.to_lowercase();
    const CREDENCIALES: [&str; 5] = [
        "authentication failed",
        "could not read username",
        "permission denied (publickey)",
        "terminal prompts disabled",
        "403 forbidden",
    ];

    if CREDENCIALES
        .iter()
        .any(|fragmento| mensaje.contains(fragmento))
    {
        ERROR_CREDENCIALES.to_string()
    } else {
        ERROR_REPOSITORIO.to_string()
    }
}

fn recoger_lotes(raiz: &Path) -> Result<Vec<LoteRecibido>, String> {
    let mut lotes = Vec::new();
    recoger_en(raiz, raiz, &mut lotes)?;
    lotes.sort_by(|a, b| {
        a.dispositivo
            .cmp(&b.dispositivo)
            .then(a.contenido.cmp(&b.contenido))
    });
    Ok(lotes)
}

fn recoger_en(raiz: &Path, directorio: &Path, lotes: &mut Vec<LoteRecibido>) -> Result<(), String> {
    let entradas = fs::read_dir(directorio).map_err(|_| ERROR_REPOSITORIO.to_string())?;
    for entrada in entradas {
        let entrada = entrada.map_err(|_| ERROR_REPOSITORIO.to_string())?;
        if entrada.file_name().to_string_lossy() == ".git" {
            continue;
        }

        let ruta = entrada.path();
        if ruta.is_dir() {
            recoger_en(raiz, &ruta, lotes)?;
        } else if ruta
            .extension()
            .is_some_and(|extension| extension == "ccbak")
        {
            let texto =
                String::from_utf8(fs::read(&ruta).map_err(|_| ERROR_REPOSITORIO.to_string())?)
                    .map_err(|_| ERROR_REPOSITORIO.to_string())?;
            let limpio: String = texto.chars().filter(|c| *c != '\r' && *c != '\n').collect();
            let contenido = STANDARD
                .decode(limpio)
                .map_err(|_| ERROR_REPOSITORIO.to_string())?;

            lotes.push(LoteRecibido {
                dispositivo: dispositivo_de_ruta(raiz, &ruta),
                contenido,
            });
        }
    }
    Ok(())
}

fn dispositivo_de_ruta(raiz: &Path, ruta: &Path) -> String {
    let relativo = ruta.strip_prefix(raiz).unwrap_or(ruta);
    let mut componentes = relativo.components();
    match (componentes.next(), componentes.next()) {
        (Some(primero), Some(_)) => primero.as_os_str().to_string_lossy().into_owned(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    struct DirTemporal(PathBuf);

    impl DirTemporal {
        fn nueva() -> Self {
            let ruta = std::env::temp_dir().join(format!("cc-git-{}", Uuid::new_v4()));
            fs::create_dir_all(&ruta).unwrap();
            Self(ruta)
        }

        fn ruta(&self, nombre: &str) -> PathBuf {
            self.0.join(nombre)
        }
    }

    impl Drop for DirTemporal {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn git_en(directorio: &Path, argumentos: &[&str]) -> String {
        let salida = Command::new("git")
            .arg("-C")
            .arg(directorio)
            .args(argumentos)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .unwrap();
        assert!(
            salida.status.success(),
            "git {:?} falló: {}",
            argumentos,
            String::from_utf8_lossy(&salida.stderr)
        );
        String::from_utf8_lossy(&salida.stdout).into_owned()
    }

    fn configurar_identidad(directorio: &Path) {
        git_en(
            directorio,
            &["config", "user.email", "pruebas@cuentasclaras.local"],
        );
        git_en(directorio, &["config", "user.name", "Prueba"]);
    }

    #[test]
    fn publica_y_lee_lotes_entre_dispositivos_sin_reescribirlos() {
        let dir = DirTemporal::nueva();
        let remoto = dir.ruta("remoto.git");
        let remoto_texto = remoto.to_string_lossy().into_owned();
        git_en(
            &dir.0,
            &["init", "--bare", "--quiet", remoto_texto.as_str()],
        );

        let ruta_a = dir.ruta("dispositivo-a");
        let transporte_a = TransporteGit::new(remoto_texto.clone(), ruta_a.clone());
        transporte_a.asegurar_clon().unwrap();
        configurar_identidad(&ruta_a);
        transporte_a.publicar_lote("disp-a", b"lote-a").unwrap();

        let ruta_b = dir.ruta("dispositivo-b");
        let transporte_b = TransporteGit::new(remoto_texto, ruta_b.clone());
        transporte_b.asegurar_clon().unwrap();
        configurar_identidad(&ruta_b);

        let lotes = transporte_b.leer_lotes().unwrap();
        assert_eq!(lotes.len(), 1);
        assert_eq!(lotes[0].dispositivo, "disp-a");
        assert_eq!(lotes[0].contenido, b"lote-a");

        transporte_b.publicar_lote("disp-b", b"lote-b").unwrap();

        let lotes = transporte_a.leer_lotes().unwrap();
        assert_eq!(lotes.len(), 2);
        assert!(
            lotes
                .iter()
                .any(|lote| lote.dispositivo == "disp-a" && lote.contenido == b"lote-a")
        );
        assert!(
            lotes
                .iter()
                .any(|lote| lote.dispositivo == "disp-b" && lote.contenido == b"lote-b")
        );

        assert_eq!(transporte_a.leer_lotes().unwrap().len(), 2);
        assert_eq!(transporte_b.leer_lotes().unwrap().len(), 2);
    }

    #[test]
    fn lee_lotes_con_el_remoto_vacio_y_luego_con_un_clon_sin_commits() {
        let dir = DirTemporal::nueva();
        let remoto = dir.ruta("remoto.git");
        let remoto_texto = remoto.to_string_lossy().into_owned();
        git_en(
            &dir.0,
            &["init", "--bare", "--quiet", remoto_texto.as_str()],
        );

        let clon_vacio = dir.ruta("clon-vacio");
        let transporte_vacio = TransporteGit::new(remoto_texto.clone(), clon_vacio.clone());
        assert!(transporte_vacio.leer_lotes().unwrap().is_empty());

        let ruta_a = dir.ruta("dispositivo-a");
        let transporte_a = TransporteGit::new(remoto_texto, ruta_a.clone());
        transporte_a.asegurar_clon().unwrap();
        configurar_identidad(&ruta_a);
        transporte_a.publicar_lote("disp-a", b"lote-a").unwrap();

        // Ese mismo clon sigue sin commits propios: debe adoptar la rama del remoto.
        let lotes = transporte_vacio.leer_lotes().unwrap();
        assert_eq!(lotes.len(), 1);
        assert_eq!(lotes[0].dispositivo, "disp-a");
        assert_eq!(lotes[0].contenido, b"lote-a");
    }

    #[test]
    fn la_cabecera_acepta_token_y_usuario_token() {
        assert_eq!(
            cabecera_autorizacion("ghp_token"),
            format!(
                "Authorization: Basic {}",
                STANDARD.encode("x-access-token:ghp_token")
            )
        );
        assert_eq!(
            cabecera_autorizacion("usuario:ghp_token"),
            format!(
                "Authorization: Basic {}",
                STANDARD.encode("usuario:ghp_token")
            )
        );
    }

    #[test]
    fn las_credenciales_solo_se_inyectan_en_urls_http() {
        let ssh = TransporteGit::new("git@github.com:usuario/libro.git", "/tmp/clon")
            .con_credenciales(Some("ghp_token".to_string()));
        assert!(ssh.cabecera().is_none());

        let https = TransporteGit::new("https://github.com/usuario/libro.git", "/tmp/clon")
            .con_credenciales(Some("ghp_token".to_string()));
        assert!(https.cabecera().is_some());
    }
}
