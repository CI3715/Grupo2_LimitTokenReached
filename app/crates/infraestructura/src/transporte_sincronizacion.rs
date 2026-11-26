use aplicacion::puertos::{LoteCifrado, TransporteSincronizacion as PuertoTransporte};
use git2::{
    build::CheckoutBuilder, Cred, CredentialType, ErrorCode, FetchOptions, PushOptions,
    RemoteCallbacks, Repository, Signature,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

const DIRECTORIO_LOTES: &str = "lotes";
const EXTENSION_LOTE: &str = "lote";
const INTENTOS_PUSH: usize = 5;

#[derive(Clone)]
pub struct CredencialesGit {
    pub usuario: String,
    pub secreto: String,
}

#[derive(Serialize, Deserialize, Default)]
struct EstadoLecturas {
    ids: HashSet<String>,
}

struct RegistroLecturas {
    ruta: PathBuf,
    ids: Mutex<HashSet<String>>,
}

impl RegistroLecturas {
    fn abrir(ruta: PathBuf) -> Result<Self, String> {
        let ids = match File::open(&ruta) {
            Ok(mut archivo) => {
                let mut contenido = String::new();
                archivo.read_to_string(&mut contenido).map_err(|error| {
                    format!("No se pudo leer el estado local de sincronización: {error}")
                })?;
                serde_json::from_str::<EstadoLecturas>(&contenido)
                    .map_err(|error| {
                        format!("El estado local de sincronización está dañado: {error}")
                    })?
                    .ids
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashSet::new(),
            Err(error) => {
                return Err(format!("No se pudo leer el estado local de sincronización: {error}"));
            }
        };
        Ok(Self {
            ruta,
            ids: Mutex::new(ids),
        })
    }

    fn contiene(&self, id: &str) -> Result<bool, String> {
        self.ids
            .lock()
            .map(|ids| ids.contains(id))
            .map_err(|error| format!("No se pudo consultar el estado de sincronización: {error}"))
    }

    fn marcar(&self, nuevos_ids: &[String]) -> Result<(), String> {
        let mut ids = self.ids.lock().map_err(|error| {
            format!("No se pudo actualizar el estado de sincronización: {error}")
        })?;
        let ids_actualizados = ids
            .iter()
            .cloned()
            .chain(nuevos_ids.iter().cloned())
            .collect::<HashSet<_>>();
        let contenido = serde_json::to_vec(&EstadoLecturas {
            ids: ids_actualizados.clone(),
        })
        .map_err(|error| format!("No se pudo serializar el estado local: {error}"))?;
        escribir_atomico(&self.ruta, &contenido)?;
        *ids = ids_actualizados;
        Ok(())
    }
}

pub struct TransporteCarpeta {
    raiz: PathBuf,
    dispositivo: String,
    lecturas: RegistroLecturas,
}

impl TransporteCarpeta {
    pub fn nuevo(
        raiz: impl Into<PathBuf>,
        archivo_estado_local: impl Into<PathBuf>,
        dispositivo: impl Into<String>,
    ) -> Result<Self, String> {
        let raiz = raiz.into();
        let dispositivo = dispositivo.into();
        validar_segmento(&dispositivo, "identificador del dispositivo")?;
        fs::create_dir_all(raiz.join(DIRECTORIO_LOTES))
            .map_err(|error| format!("No se pudo abrir la carpeta compartida: {error}"))?;
        Ok(Self {
            raiz,
            dispositivo,
            lecturas: RegistroLecturas::abrir(archivo_estado_local.into())?,
        })
    }

    fn listar_archivos(&self) -> Result<Vec<(String, PathBuf)>, String> {
        let raiz_lotes = self.raiz.join(DIRECTORIO_LOTES);
        let dispositivos = fs::read_dir(&raiz_lotes)
            .map_err(|error| format!("No se pudo leer la carpeta compartida: {error}"))?;
        let mut lotes = Vec::new();
        for dispositivo in dispositivos {
            let dispositivo = dispositivo.map_err(|error| error.to_string())?;
            if !dispositivo
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                continue;
            }
            let nombre_dispositivo = dispositivo.file_name().to_string_lossy().into_owned();
            validar_segmento(&nombre_dispositivo, "identificador del dispositivo")?;
            if nombre_dispositivo == self.dispositivo {
                continue;
            }
            for archivo in fs::read_dir(dispositivo.path()).map_err(|error| error.to_string())? {
                let archivo = archivo.map_err(|error| error.to_string())?;
                if !archivo
                    .file_type()
                    .map_err(|error| error.to_string())?
                    .is_file()
                    || archivo.path().extension().and_then(|ext| ext.to_str())
                        != Some(EXTENSION_LOTE)
                {
                    continue;
                }
                let nombre = archivo
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .ok_or_else(|| "El lote tiene un nombre inválido.".to_string())?
                    .to_owned();
                validar_segmento(&nombre, "identificador del lote")?;
                lotes.push((format!("{nombre_dispositivo}/{nombre}"), archivo.path()));
            }
        }
        lotes.sort_by(|izquierda, derecha| izquierda.0.cmp(&derecha.0));
        Ok(lotes)
    }
}

impl PuertoTransporte for TransporteCarpeta {
    fn leer_lotes_nuevos(&self) -> Result<Vec<LoteCifrado>, String> {
        let mut lotes = Vec::new();
        for (id, ruta) in self.listar_archivos()? {
            if self.lecturas.contiene(&id)? {
                continue;
            }
            let dispositivo = id
                .split_once('/')
                .map(|(dispositivo, _)| dispositivo)
                .ok_or_else(|| "Identificador de lote inválido.".to_string())?
                .to_owned();
            let contenido = fs::read(&ruta)
                .map_err(|error| format!("No se pudo leer un lote compartido: {error}"))?;
            lotes.push(LoteCifrado {
                id,
                dispositivo,
                contenido,
            });
        }
        Ok(lotes)
    }

    fn marcar_lotes_leidos(&self, ids: &[String]) -> Result<(), String> {
        self.lecturas.marcar(ids)
    }

    fn escribir_lote(&self, lote: &LoteCifrado) -> Result<(), String> {
        if lote.dispositivo != self.dispositivo {
            return Err("Un dispositivo solo puede escribir en su propia carpeta.".to_string());
        }
        let id_lote = validar_id_lote(&lote.id, &self.dispositivo)?;
        let directorio = self.raiz.join(DIRECTORIO_LOTES).join(&self.dispositivo);
        fs::create_dir_all(&directorio)
            .map_err(|error| format!("No se pudo crear la carpeta del dispositivo: {error}"))?;
        let destino = directorio.join(format!("{id_lote}.{EXTENSION_LOTE}"));
        if destino.exists() {
            return Err("Los lotes son inmutables y ese identificador ya existe.".to_string());
        }
        escribir_atomico(&destino, &lote.contenido)
    }
}

#[derive(Default)]
pub struct EstadoMemoria {
    lotes: HashMap<String, LoteCifrado>,
    leidos: HashMap<String, HashSet<String>>,
}

#[derive(Clone)]
pub struct TransporteMemoria {
    dispositivo: String,
    estado: Arc<Mutex<EstadoMemoria>>,
}

impl TransporteMemoria {
    pub fn nuevo(
        dispositivo: impl Into<String>,
        estado: Arc<Mutex<EstadoMemoria>>,
    ) -> Result<Self, String> {
        let dispositivo = dispositivo.into();
        validar_segmento(&dispositivo, "identificador del dispositivo")?;
        Ok(Self {
            dispositivo,
            estado,
        })
    }

    pub fn nuevo_destino_compartido() -> Arc<Mutex<EstadoMemoria>> {
        Arc::new(Mutex::new(EstadoMemoria::default()))
    }
}

impl PuertoTransporte for TransporteMemoria {
    fn leer_lotes_nuevos(&self) -> Result<Vec<LoteCifrado>, String> {
        let estado = self
            .estado
            .lock()
            .map_err(|error| format!("No se pudo acceder al transporte en memoria: {error}"))?;
        let leidos = estado.leidos.get(&self.dispositivo);
        let mut lotes = estado
            .lotes
            .values()
            .filter(|lote| lote.dispositivo != self.dispositivo)
            .filter(|lote| !leidos.is_some_and(|ids| ids.contains(&lote.id)))
            .cloned()
            .collect::<Vec<_>>();
        lotes.sort_by(|izquierda, derecha| izquierda.id.cmp(&derecha.id));
        Ok(lotes)
    }

    fn marcar_lotes_leidos(&self, ids: &[String]) -> Result<(), String> {
        let mut estado = self
            .estado
            .lock()
            .map_err(|error| format!("No se pudo actualizar el transporte en memoria: {error}"))?;
        let leidos = estado.leidos.entry(self.dispositivo.clone()).or_default();
        leidos.extend(ids.iter().cloned());
        Ok(())
    }

    fn escribir_lote(&self, lote: &LoteCifrado) -> Result<(), String> {
        if lote.dispositivo != self.dispositivo {
            return Err("Un dispositivo solo puede escribir sus propios lotes.".to_string());
        }
        validar_id_lote(&lote.id, &self.dispositivo)?;
        let llave = lote.id.clone();
        let mut estado = self
            .estado
            .lock()
            .map_err(|error| format!("No se pudo escribir en el transporte en memoria: {error}"))?;
        if estado.lotes.contains_key(&llave) {
            return Err("Los lotes son inmutables y ese identificador ya existe.".to_string());
        }
        estado.lotes.insert(llave, lote.clone());
        Ok(())
    }
}

pub struct TransporteGit {
    remoto: String,
    directorio: PathBuf,
    rama: String,
    dispositivo: String,
    credenciales: Option<CredencialesGit>,
    lecturas: RegistroLecturas,
}

impl TransporteGit {
    pub fn nuevo(
        remoto: impl Into<String>,
        directorio: impl Into<PathBuf>,
        archivo_estado_local: impl Into<PathBuf>,
        rama: impl Into<String>,
        dispositivo: impl Into<String>,
        credenciales: Option<CredencialesGit>,
    ) -> Result<Self, String> {
        let remoto = remoto.into();
        let directorio = directorio.into();
        let rama = rama.into();
        let dispositivo = dispositivo.into();
        if !rama.is_empty() {
            validar_segmento(&rama, "nombre de rama")?;
        }
        validar_segmento(&dispositivo, "identificador del dispositivo")?;
        if remoto.trim().is_empty() || remoto_contiene_credenciales(&remoto) {
            return Err("La URL del repositorio no puede incluir credenciales.".to_string());
        }
        let mut transporte = Self {
            remoto,
            directorio,
            rama,
            dispositivo,
            credenciales,
            lecturas: RegistroLecturas::abrir(archivo_estado_local.into())?,
        };
        let repo = transporte.abrir_repositorio()?;
        if transporte.rama.is_empty() {
            transporte.rama = match repo.head() {
                Ok(head) => head
                    .shorthand()
                    .ok_or_else(|| {
                        "No se pudo determinar la rama predeterminada de Git.".to_string()
                    })?
                    .to_string(),
                Err(error) if error.code() == ErrorCode::UnbornBranch => "main".to_string(),
                Err(_) => {
                    return Err("No se pudo determinar la rama predeterminada de Git.".to_string());
                }
            };
            validar_segmento(&transporte.rama, "nombre de rama")?;
            if repo.head().is_err() {
                repo.set_head(&format!("refs/heads/{}", transporte.rama))
                    .map_err(|_| "No se pudo inicializar la rama local de Git.".to_string())?;
            }
        }
        Ok(transporte)
    }

    fn callbacks(&self) -> RemoteCallbacks<'_> {
        let credenciales = self.credenciales.clone();
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, usuario_url, tipos| {
            if tipos.contains(CredentialType::USER_PASS_PLAINTEXT)
                && let Some(credenciales) = &credenciales
            {
                return Cred::userpass_plaintext(&credenciales.usuario, &credenciales.secreto);
            }
            if tipos.contains(CredentialType::SSH_KEY) {
                if let Some(usuario) = usuario_url {
                    return Cred::ssh_key_from_agent(usuario);
                }
            }
            Cred::default()
        });
        callbacks
    }

    fn abrir_repositorio(&self) -> Result<Repository, String> {
        if self.directorio.join(".git").exists() {
            let repo = Repository::open(&self.directorio)
                .map_err(|error| format!("No se pudo abrir el repositorio Git: {error}"))?;
            self.verificar_remoto(&repo)?;
            Ok(repo)
        } else {
            if let Some(padre) = self.directorio.parent() {
                fs::create_dir_all(padre).map_err(|error| {
                    format!("No se pudo preparar la carpeta local de Git: {error}")
                })?;
            }
            let mut opciones = git2::build::RepoBuilder::new();
            if !self.rama.is_empty() {
                opciones.branch(&self.rama);
            }
            let mut fetch = FetchOptions::new();
            fetch.remote_callbacks(self.callbacks());
            opciones.fetch_options(fetch);
            opciones
                .clone(&self.remoto, &self.directorio)
                .map_err(|error| formatear_error_git(error, "No se pudo clonar el repositorio"))
        }
    }

    fn verificar_remoto(&self, repo: &Repository) -> Result<(), String> {
        let remoto = repo
            .find_remote("origin")
            .map_err(|error| format!("El repositorio Git no tiene un remoto origin: {error}"))?;
        if remoto.url() != Some(self.remoto.as_str()) {
            return Err("El repositorio local está asociado a un destino Git distinto.".to_string());
        }
        Ok(())
    }

    fn actualizar_desde_remoto(&self, repo: &Repository) -> Result<(), String> {
        let mut remoto = repo
            .find_remote("origin")
            .map_err(|error| format!("No se encontró el remoto Git: {error}"))?;
        let refspec = format!(
            "+refs/heads/{}:refs/remotes/origin/{}",
            self.rama, self.rama
        );
        let mut opciones = FetchOptions::new();
        opciones.remote_callbacks(self.callbacks());
        if let Err(error) = remoto.fetch(&[&refspec], Some(&mut opciones), None) {
            if error.code() != ErrorCode::NotFound {
                return Err(formatear_error_git(
                    error,
                    "No se pudo traer cambios del repositorio",
                ));
            }
        }

        let referencia_remota = format!("refs/remotes/origin/{}", self.rama);
        let oid_remoto = match repo.refname_to_id(&referencia_remota) {
            Ok(oid) => oid,
            Err(error) if error.code() == ErrorCode::NotFound => return Ok(()),
            Err(error) => return Err(format!("No se pudo encontrar la rama remota: {error}")),
        };
        let oid_local = match repo.head().and_then(|head| head.target().ok_or_else(|| {
            git2::Error::from_str("La rama local no apunta a un commit")
        })) {
            Ok(oid) => oid,
            Err(error) if error.code() == ErrorCode::UnbornBranch => {
                let referencia_local = format!("refs/heads/{}", self.rama);
                repo.reference(&referencia_local, oid_remoto, true, "Inicializar rama local")
                    .map_err(|error| error.to_string())?;
                repo.set_head(&referencia_local)
                    .map_err(|error| error.to_string())?;
                repo.checkout_head(Some(CheckoutBuilder::new().force()))
                    .map_err(|error| error.to_string())?;
                return Ok(());
            }
            Err(error) => return Err(format!("No se pudo leer la rama local: {error}")),
        };
        if oid_local == oid_remoto {
            return Ok(());
        }

        let remoto = repo
            .find_annotated_commit(oid_remoto)
            .map_err(|error| error.to_string())?;
        let (analisis, _) = repo
            .merge_analysis(&[&remoto])
            .map_err(|error| format!("No se pudo analizar la rama remota: {error}"))?;

        if analisis.is_up_to_date() {
            return Ok(());
        }
        if analisis.is_fast_forward() {
            let referencia_local = format!("refs/heads/{}", self.rama);
            let mut referencia = repo
                .find_reference(&referencia_local)
                .or_else(|_| {
                    repo.reference(&referencia_local, oid_remoto, true, "Crear rama local")
                })
                .map_err(|error| error.to_string())?;
            referencia
                .set_target(oid_remoto, "Avance rápido desde remoto")
                .map_err(|error| error.to_string())?;
            repo.set_head(&referencia_local)
                .map_err(|error| error.to_string())?;
            repo.checkout_head(Some(CheckoutBuilder::new().force()))
                .map_err(|error| error.to_string())?;
            return Ok(());
        }

        repo.merge(&[&remoto], None, None)
            .map_err(|error| format!("No se pudieron fusionar los cambios remotos: {error}"))?;
        let mut indice = repo.index().map_err(|error| error.to_string())?;
        if indice.has_conflicts() {
            repo.cleanup_state().map_err(|error| error.to_string())?;
            return Err(
                "El repositorio tiene conflictos que no pueden fusionarse automáticamente."
                    .to_string(),
            );
        }
        let oid_arbol = indice.write_tree().map_err(|error| error.to_string())?;
        let arbol = repo.find_tree(oid_arbol).map_err(|error| error.to_string())?;
        let commit_local = repo
            .find_commit(oid_local)
            .map_err(|error| error.to_string())?;
        let commit_remoto = repo
            .find_commit(oid_remoto)
            .map_err(|error| error.to_string())?;
        let firma = self.firma(repo)?;
        repo.commit(
            Some("HEAD"),
            &firma,
            &firma,
            "Fusionar cambios remotos de Cuentas Claras",
            &arbol,
            &[&commit_local, &commit_remoto],
        )
        .map_err(|error| format!("No se pudo confirmar la fusión remota: {error}"))?;
        repo.cleanup_state().map_err(|error| error.to_string())?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .map_err(|error| error.to_string())
    }

    fn firma(&self, repo: &Repository) -> Result<Signature<'static>, String> {
        repo.signature()
            .or_else(|_| Signature::now("Cuentas Claras", "cuentas-claras@localhost"))
            .map_err(|error| format!("No se pudo crear la identidad del commit Git: {error}"))
    }

    fn listar_archivos(&self, repo: &Repository) -> Result<Vec<(String, PathBuf)>, String> {
        let raiz = self.directorio.join(DIRECTORIO_LOTES);
        if !raiz.exists() {
            return Ok(Vec::new());
        }
        let arbol = repo
            .head()
            .and_then(|head| head.peel_to_tree())
            .map_err(|error| format!("No se pudo leer el estado del repositorio Git: {error}"))?;
        let mut lotes = Vec::new();
        for carpeta in fs::read_dir(&raiz).map_err(|error| error.to_string())? {
            let carpeta = carpeta.map_err(|error| error.to_string())?;
            if !carpeta
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                continue;
            }
            let dispositivo = carpeta.file_name().to_string_lossy().into_owned();
            validar_segmento(&dispositivo, "identificador del dispositivo")?;
            if dispositivo == self.dispositivo {
                continue;
            }
            for archivo in fs::read_dir(carpeta.path()).map_err(|error| error.to_string())? {
                let archivo = archivo.map_err(|error| error.to_string())?;
                if !archivo
                    .file_type()
                    .map_err(|error| error.to_string())?
                    .is_file()
                    || archivo.path().extension().and_then(|ext| ext.to_str())
                        != Some(EXTENSION_LOTE)
                {
                    continue;
                }
                let id_lote = archivo
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .ok_or_else(|| "El lote Git tiene un nombre inválido.".to_string())?
                    .to_owned();
                validar_segmento(&id_lote, "identificador del lote")?;
                let id = format!("{dispositivo}/{id_lote}");
                let ruta = archivo.path();
                let ruta_relativa = ruta
                    .strip_prefix(&self.directorio)
                    .map_err(|error| error.to_string())?;
                if arbol.get_path(ruta_relativa).is_ok() {
                    lotes.push((id, ruta));
                }
            }
        }
        lotes.sort_by(|izquierda, derecha| izquierda.0.cmp(&derecha.0));
        Ok(lotes)
    }

    fn push_con_reintentos(&self, repo: &Repository) -> Result<(), String> {
        for intento in 0..INTENTOS_PUSH {
            let mut remoto = repo
                .find_remote("origin")
                .map_err(|error| format!("No se encontró el remoto Git: {error}"))?;
            let refspec = format!("refs/heads/{}:refs/heads/{}", self.rama, self.rama);
            let mut opciones = PushOptions::new();
            opciones.remote_callbacks(self.callbacks());
            let resultado = remoto.push(&[&refspec], Some(&mut opciones));
            match resultado {
                Ok(()) => return Ok(()),
                Err(error)
                    if error.code() == ErrorCode::NotFastForward
                        && intento + 1 < INTENTOS_PUSH =>
                {
                    self.actualizar_desde_remoto(repo)?;
                }
                Err(error) => {
                    return Err(formatear_error_git(
                        error,
                        "No se pudieron publicar los cambios en Git",
                    ));
                }
            };
        }
        Err("El repositorio cambió repetidamente; vuelve a sincronizar.".to_string())
    }
}

impl PuertoTransporte for TransporteGit {
    fn leer_lotes_nuevos(&self) -> Result<Vec<LoteCifrado>, String> {
        let repo = self.abrir_repositorio()?;
        self.actualizar_desde_remoto(&repo)?;
        let mut lotes = Vec::new();
        for (id, ruta) in self.listar_archivos(&repo)? {
            if self.lecturas.contiene(&id)? {
                continue;
            }
            let dispositivo = id
                .split_once('/')
                .map(|(dispositivo, _)| dispositivo)
                .ok_or_else(|| "Identificador de lote inválido.".to_string())?
                .to_owned();
            let contenido = fs::read(&ruta)
                .map_err(|error| format!("No se pudo leer un lote del repositorio: {error}"))?;
            lotes.push(LoteCifrado {
                id,
                dispositivo,
                contenido,
            });
        }
        Ok(lotes)
    }

    fn marcar_lotes_leidos(&self, ids: &[String]) -> Result<(), String> {
        self.lecturas.marcar(ids)
    }

    fn escribir_lote(&self, lote: &LoteCifrado) -> Result<(), String> {
        if lote.dispositivo != self.dispositivo {
            return Err("Un dispositivo solo puede escribir en su propia carpeta.".to_string());
        }
        let id_lote = validar_id_lote(&lote.id, &self.dispositivo)?;
        let repo = self.abrir_repositorio()?;
        self.actualizar_desde_remoto(&repo)?;

        let directorio = self
            .directorio
            .join(DIRECTORIO_LOTES)
            .join(&self.dispositivo);
        fs::create_dir_all(&directorio)
            .map_err(|error| format!("No se pudo crear la carpeta local del dispositivo: {error}"))?;
        let destino = directorio.join(format!("{id_lote}.{EXTENSION_LOTE}"));
        if destino.exists() {
            return Err("Los lotes son inmutables y ese identificador ya existe.".to_string());
        }
        escribir_atomico(&destino, &lote.contenido)?;

        let mut indice = repo.index().map_err(|error| error.to_string())?;
        let ruta_relativa = destino
            .strip_prefix(&self.directorio)
            .map_err(|error| error.to_string())?;
        indice
            .add_path(ruta_relativa)
            .map_err(|error| format!("No se pudo preparar el lote para Git: {error}"))?;
        indice.write().map_err(|error| error.to_string())?;
        let oid_arbol = indice.write_tree().map_err(|error| error.to_string())?;
        let arbol = repo.find_tree(oid_arbol).map_err(|error| error.to_string())?;
        let firma = self.firma(&repo)?;
        let mensaje = "Agregar lote cifrado de Cuentas Claras";
        match repo.head().and_then(|head| head.peel_to_commit()) {
            Ok(padre) => {
                repo.commit(
                    Some("HEAD"),
                    &firma,
                    &firma,
                    mensaje,
                    &arbol,
                    &[&padre],
                )
            }
            Err(error) if error.code() == ErrorCode::UnbornBranch => {
                repo.commit(Some("HEAD"), &firma, &firma, mensaje, &arbol, &[])
            }
            Err(error) => return Err(format!("No se encontró el commit local: {error}")),
        }
        .map_err(|_| "No se pudo confirmar el lote.".to_string())?;
        self.push_con_reintentos(&repo)
    }
}

fn escribir_atomico(destino: &Path, contenido: &[u8]) -> Result<(), String> {
    let directorio = destino
        .parent()
        .ok_or_else(|| "No se pudo determinar la carpeta del lote.".to_string())?;
    fs::create_dir_all(directorio)
        .map_err(|error| format!("No se pudo crear la carpeta del lote: {error}"))?;
    if destino.exists() {
        return Err("Los lotes son inmutables y ese identificador ya existe.".to_string());
    }
    let nombre_temporal = format!(
        ".{}.tmp-{}",
        destino
            .file_name()
            .and_then(|nombre| nombre.to_str())
            .ok_or_else(|| "El lote tiene un nombre inválido.".to_string())?,
        Uuid::new_v4()
    );
    let temporal = directorio.join(nombre_temporal);
    let resultado = (|| {
        let mut archivo = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|error| format!("No se pudo crear el lote temporal: {error}"))?;
        archivo
            .write_all(contenido)
            .map_err(|error| format!("No se pudo escribir el lote temporal: {error}"))?;
        archivo
            .sync_all()
            .map_err(|error| format!("No se pudo guardar el lote temporal: {error}"))?;
        fs::rename(&temporal, destino)
            .map_err(|error| format!("No se pudo publicar el lote de forma atómica: {error}"))
    })();
    if resultado.is_err() {
        let _ = fs::remove_file(temporal);
    }
    resultado
}

fn validar_segmento(valor: &str, descripcion: &str) -> Result<(), String> {
    let ruta = Path::new(valor);
    if valor.is_empty()
        || valor == "."
        || valor == ".."
        || ruta.components().any(|componente| !matches!(componente, Component::Normal(_)))
        || valor.contains(['/', '\\'])
    {
        return Err(format!("El {descripcion} no es válido."));
    }
    Ok(())
}

fn validar_id_lote<'a>(id: &'a str, dispositivo: &str) -> Result<&'a str, String> {
    let (dispositivo_id, id_lote) = id
        .split_once('/')
        .ok_or_else(|| "El identificador del lote debe incluir su dispositivo.".to_string())?;
    if dispositivo_id != dispositivo {
        return Err("Un dispositivo solo puede escribir en su propia carpeta.".to_string());
    }
    validar_segmento(id_lote, "identificador del lote")?;
    Ok(id_lote)
}

fn remoto_contiene_credenciales(remoto: &str) -> bool {
    remoto
        .split_once("://")
        .and_then(|(_, resto)| resto.split('/').next())
        .is_some_and(|autoridad| autoridad.contains('@'))
}

fn formatear_error_git(error: git2::Error, contexto: &str) -> String {
    if error.code() == ErrorCode::Auth || error.class() == git2::ErrorClass::Ssh {
        return "Las credenciales de Git fueron rechazadas; actualízalas en la configuración de sincronización."
            .to_string();
    }
    if error.code() == ErrorCode::NotFound || error.class() == git2::ErrorClass::Net {
        return "No se pudo llegar al repositorio Git; se conservarán los cambios pendientes."
            .to_string();
    }
    format!("{contexto}.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{process::Command, time::SystemTime};

    fn directorio_temporal(nombre: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cuentas-claras-{nombre}-{nonce}"))
    }

    fn lote(device: &str, id: &str, contenido: &[u8]) -> LoteCifrado {
        LoteCifrado {
            id: format!("{device}/{id}"),
            dispositivo: device.to_string(),
            contenido: contenido.to_vec(),
        }
    }

    fn contrato_transporte(primero: &dyn PuertoTransporte, segundo: &dyn PuertoTransporte) {
        let original = lote("device-a", "batch-1", b"encrypted");
        primero.escribir_lote(&original).unwrap();
        let recibidos = segundo.leer_lotes_nuevos().unwrap();
        assert_eq!(recibidos.len(), 1);
        assert_eq!(recibidos[0].id, original.id);
        assert_eq!(recibidos[0].dispositivo, original.dispositivo);
        assert_eq!(recibidos[0].contenido, original.contenido);
        segundo
            .marcar_lotes_leidos(&recibidos.iter().map(|lote| lote.id.clone()).collect::<Vec<_>>())
            .unwrap();
        assert!(segundo.leer_lotes_nuevos().unwrap().is_empty());
        assert!(primero.escribir_lote(&original).is_err());
        assert!(segundo.escribir_lote(&lote("device-a", "batch-2", b"bad")).is_err());
    }

    #[test]
    fn transporte_memoria_cumple_el_contrato() {
        let estado = TransporteMemoria::nuevo_destino_compartido();
        let primero = TransporteMemoria::nuevo("device-a", estado.clone()).unwrap();
        let segundo = TransporteMemoria::nuevo("device-b", estado).unwrap();
        contrato_transporte(&primero, &segundo);
    }

    #[test]
    fn transporte_carpeta_cumple_el_contrato_y_persiste_lecturas_locales() {
        let directorio = directorio_temporal("folder");
        let primero = TransporteCarpeta::nuevo(
            directorio.join("compartido"),
            directorio.join("local-a/estado.json"),
            "device-a",
        )
        .unwrap();
        let segundo = TransporteCarpeta::nuevo(
            directorio.join("compartido"),
            directorio.join("local-b/estado.json"),
            "device-b",
        )
        .unwrap();
        contrato_transporte(&primero, &segundo);
        let segundo_reabierto = TransporteCarpeta::nuevo(
            directorio.join("compartido"),
            directorio.join("local-b/estado.json"),
            "device-b",
        )
        .unwrap();
        assert!(segundo_reabierto.leer_lotes_nuevos().unwrap().is_empty());
        let lote_path = directorio.join("compartido/lotes/device-a/batch-1.lote");
        assert_eq!(fs::read(lote_path).unwrap(), b"encrypted");
        fs::remove_dir_all(directorio).unwrap();
    }

    #[test]
    fn transporte_git_cumple_el_contrato_en_repositorio_local() {
        let directorio = directorio_temporal("git");
        let remoto = directorio.join("remoto.git");
        let local_a = directorio.join("local-a");
        let local_b = directorio.join("local-b");
        fs::create_dir_all(&directorio).unwrap();
        let salida = Command::new("git")
            .args(["init", "--bare", remoto.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(salida.status.success());

        let seed = directorio.join("seed");
        let salida = Command::new("git")
            .args(["init", "-b", "main", seed.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(salida.status.success());
        Command::new("git")
            .args(["-C", seed.to_str().unwrap(), "config", "user.name", "Test"])
            .status()
            .unwrap();
        Command::new("git")
            .args(["-C", seed.to_str().unwrap(), "config", "user.email", "test@example.com"])
            .status()
            .unwrap();
        fs::write(seed.join("README"), "seed").unwrap();
        Command::new("git")
            .args(["-C", seed.to_str().unwrap(), "add", "README"])
            .status()
            .unwrap();
        Command::new("git")
            .args(["-C", seed.to_str().unwrap(), "commit", "-m", "seed"])
            .status()
            .unwrap();
        Command::new("git")
            .args(["-C", seed.to_str().unwrap(), "remote", "add", "origin", remoto.to_str().unwrap()])
            .status()
            .unwrap();
        Command::new("git")
            .args(["-C", seed.to_str().unwrap(), "push", "-u", "origin", "main"])
            .status()
            .unwrap();
        Command::new("git")
            .args([
                "--git-dir",
                remoto.to_str().unwrap(),
                "symbolic-ref",
                "HEAD",
                "refs/heads/main",
            ])
            .status()
            .unwrap();

        let primero = TransporteGit::nuevo(
            remoto.to_str().unwrap(),
            &local_a,
            directorio.join("state-a/seen.json"),
            "main",
            "device-a",
            None,
        )
        .unwrap();
        let segundo = TransporteGit::nuevo(
            remoto.to_str().unwrap(),
            &local_b,
            directorio.join("state-b/seen.json"),
            "",
            "device-b",
            None,
        )
        .unwrap();
        contrato_transporte(&primero, &segundo);
        assert!(segundo.leer_lotes_nuevos().unwrap().is_empty());
        assert_eq!(
            fs::read(local_b.join("lotes/device-a/batch-1.lote")).unwrap(),
            b"encrypted"
        );
        fs::remove_dir_all(directorio).unwrap();
    }

    #[test]
    fn transporte_git_inicializa_un_repositorio_remoto_vacio() {
        let directorio = directorio_temporal("git-vacio");
        let remoto = directorio.join("remoto.git");
        let salida = Command::new("git")
            .args([
                "init",
                "--bare",
                "--initial-branch=main",
                remoto.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(salida.status.success());

        let primero = TransporteGit::nuevo(
            remoto.to_str().unwrap(),
            directorio.join("local-a"),
            directorio.join("state-a/seen.json"),
            "",
            "device-a",
            None,
        )
        .unwrap();
        primero
            .escribir_lote(&lote("device-a", "batch-1", b"encrypted"))
            .unwrap();
        let segundo = TransporteGit::nuevo(
            remoto.to_str().unwrap(),
            directorio.join("local-b"),
            directorio.join("state-b/seen.json"),
            "",
            "device-b",
            None,
        )
        .unwrap();
        let recibidos = segundo.leer_lotes_nuevos().unwrap();
        assert_eq!(recibidos.len(), 1);
        assert_eq!(recibidos[0].id, "device-a/batch-1");
        fs::remove_dir_all(directorio).unwrap();
    }

    #[test]
    fn rechaza_identificadores_que_intentan_salir_de_su_carpeta() {
        assert!(validar_segmento("../otra-carpeta", "identificador de lote").is_err());
        assert!(validar_segmento("device/lote", "identificador de lote").is_err());
    }
}
