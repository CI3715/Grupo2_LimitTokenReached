//! Comandos Tauri del respaldo cifrado y de la sincronización del libro por carpeta y Git.
//!
//! Solo exponen los casos de uso de la Entrega 8. La interfaz (doble entrada de la
//! frase, advertencia de irrecuperabilidad y botón “Sincronizar ahora”) queda fuera
//! de este crate.

use std::path::{Path, PathBuf};

use aplicacion::puertos::{
    CifradorRespaldo, Reloj, ReporteSincronizacionLibro, RepositorioEventos, SerializadorLibro,
};
use contrato::respaldo_sincronizacion::{EventoRechazadoDTO, ReporteSincronizacionDTO};
use contrato::{
    ConfigDestinoSyncDTO, ExportarLibroRequest, ExportarLibroResponse, ImportarLibroRequest,
    ImportarLibroResponse, ResultadoSyncDTO,
};
use dominio::libro::{EventoRechazado, ResumenFusion};
use infraestructura::{TransporteCarpeta, TransporteGit};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, serde::Serialize)]
enum CodigoErrorRespaldo {
    FraseIncorrectaOArchivoDanado,
    VersionFormatoNoSoportada,
    DestinoInaccesibleOSinConexion,
    CredencialesGitInvalidas,
    EventoConflictoContenido,
    FraseDemasiadoCorta,
    ErrorOperacion,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorRespaldoDTO {
    codigo: CodigoErrorRespaldo,
}

fn error_respaldo(error: String) -> ErrorRespaldoDTO {
    let codigo = match error.as_str() {
        mensaje if mensaje.starts_with("Frase de paso incorrecta o archivo dañado") => {
            CodigoErrorRespaldo::FraseIncorrectaOArchivoDanado
        }
        mensaje if mensaje.starts_with("Versión de formato no soportada") => {
            CodigoErrorRespaldo::VersionFormatoNoSoportada
        }
        mensaje if mensaje.starts_with("Las credenciales fueron rechazadas") => {
            CodigoErrorRespaldo::CredencialesGitInvalidas
        }
        mensaje
            if mensaje.starts_with("No se pudo llegar a la carpeta")
                || mensaje.starts_with("No se pudo llegar al repositorio") =>
        {
            CodigoErrorRespaldo::DestinoInaccesibleOSinConexion
        }
        mensaje if mensaje.starts_with("Evento con identificador repetido y contenido distinto") => {
            CodigoErrorRespaldo::EventoConflictoContenido
        }
        mensaje if mensaje.starts_with("La frase de paso debe tener al menos 12 caracteres") => {
            CodigoErrorRespaldo::FraseDemasiadoCorta
        }
        _ => CodigoErrorRespaldo::ErrorOperacion,
    };
    ErrorRespaldoDTO { codigo }
}

fn mapear_rechazados(rechazados: &[EventoRechazado]) -> Vec<EventoRechazadoDTO> {
    rechazados
        .iter()
        .map(|evento| EventoRechazadoDTO {
            id_evento: evento.id.clone(),
            motivo: evento.motivo.clone(),
        })
        .collect()
}

fn mapear_importacion(resumen: &ResumenFusion, fecha: String) -> ReporteSincronizacionDTO {
    ReporteSincronizacionDTO {
        eventos_nuevos: resumen.insertados as u64,
        eventos_ya_existentes: resumen.ignorados_duplicados as u64,
        eventos_rechazados: mapear_rechazados(&resumen.rechazados),
        fecha_ultima_sincronizacion: fecha,
    }
}

fn mapear_sincronizacion(reporte: &ReporteSincronizacionLibro) -> ResultadoSyncDTO {
    ResultadoSyncDTO {
        exitoso: true,
        eventos_enviados: reporte.eventos_enviados as u64,
        eventos_recibidos: reporte.eventos_recibidos as u64,
        eventos_rechazados: mapear_rechazados(&reporte.rechazados),
        fecha_ultima_sync: reporte.fecha_ultima_sincronizacion.clone(),
    }
}

/// Exporta el libro filtrado. Si `cifrar` es `false`, escribe el formato plano;
/// en ese caso la frase se ignora y la interfaz debe advertir que el archivo
/// contiene información confidencial.
fn exportar_libro_impl(
    datos: ExportarLibroRequest,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
    reloj: &dyn Reloj,
) -> Result<ExportarLibroResponse, String> {
    if datos.ruta_destino.trim().is_empty() {
        return Err("La ruta de destino no puede estar vacía.".to_string());
    }

    let ruta = PathBuf::from(&datos.ruta_destino);
    let cantidad_eventos = if datos.cifrar {
        let frase = datos
            .frase_de_paso
            .ok_or_else(|| "La frase de paso es obligatoria para exportar cifrado.".to_string())?;

        aplicacion::casos_uso::exportar_respaldo_cifrado::ejecutar(
            &ruta,
            &frase,
            repo_eventos,
            serializador,
            cifrador,
            reloj,
        )?
    } else {
        aplicacion::casos_uso::exportar_respaldo_cifrado::ejecutar_plano(
            &ruta,
            repo_eventos,
            serializador,
            reloj,
        )?
    };

    Ok(ExportarLibroResponse {
        exitoso: true,
        ruta_archivo: datos.ruta_destino,
        cantidad_eventos: cantidad_eventos as u64,
    })
}

#[tauri::command]
pub fn exportar_libro(
    datos: ExportarLibroRequest,
    repo_eventos: State<'_, Box<dyn RepositorioEventos>>,
    serializador: State<'_, Box<dyn SerializadorLibro>>,
    cifrador: State<'_, Box<dyn CifradorRespaldo>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<ExportarLibroResponse, ErrorRespaldoDTO> {
    exportar_libro_impl(
        datos,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    ).map_err(error_respaldo)
}

/// Importa un respaldo cifrado o plano. El caso de uso valida todo el archivo
/// antes de abrir la transacción de fusión, por lo que un archivo inválido no
/// modifica el libro.
fn importar_libro_impl(
    datos: ImportarLibroRequest,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
    reloj: &dyn Reloj,
) -> Result<ImportarLibroResponse, String> {
    if datos.ruta_archivo.trim().is_empty() {
        return Err("La ruta del archivo no puede estar vacía.".to_string());
    }

    let resumen = aplicacion::casos_uso::importar_respaldo_cifrado::ejecutar(
        &PathBuf::from(&datos.ruta_archivo),
        &datos.frase_de_paso,
        repo_eventos,
        serializador,
        cifrador,
    )?;

    Ok(ImportarLibroResponse {
        exitoso: true,
        reporte: mapear_importacion(&resumen, reloj.ahora()),
    })
}

#[tauri::command]
pub fn importar_libro(
    datos: ImportarLibroRequest,
    repo_eventos: State<'_, Box<dyn RepositorioEventos>>,
    serializador: State<'_, Box<dyn SerializadorLibro>>,
    cifrador: State<'_, Box<dyn CifradorRespaldo>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<ImportarLibroResponse, ErrorRespaldoDTO> {
    importar_libro_impl(
        datos,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    ).map_err(error_respaldo)
}

/// Sincroniza el libro con una carpeta compartida: trae los lotes `*.ccbak`,
/// fusiona los eventos recibidos y publica un lote nuevo e inmutable con los
/// eventos locales que aún no se habían enviado.
fn sincronizar_carpeta_libro_impl(
    ruta_carpeta: String,
    frase_de_paso: String,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
    reloj: &dyn Reloj,
) -> Result<ResultadoSyncDTO, String> {
    if ruta_carpeta.trim().is_empty() {
        return Err("La ruta de la carpeta compartida no puede estar vacía.".to_string());
    }

    let transporte = TransporteCarpeta::new(PathBuf::from(&ruta_carpeta));
    let reporte = aplicacion::casos_uso::sincronizar_libro::ejecutar(
        &transporte,
        &frase_de_paso,
        repo_eventos,
        serializador,
        cifrador,
        reloj,
    )?;

    Ok(mapear_sincronizacion(&reporte))
}

#[tauri::command]
pub fn sincronizar_carpeta_libro(
    ruta_carpeta: String,
    frase_de_paso: String,
    repo_eventos: State<'_, Box<dyn RepositorioEventos>>,
    serializador: State<'_, Box<dyn SerializadorLibro>>,
    cifrador: State<'_, Box<dyn CifradorRespaldo>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<ResultadoSyncDTO, ErrorRespaldoDTO> {
    sincronizar_carpeta_libro_impl(
        ruta_carpeta,
        frase_de_paso,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    ).map_err(error_respaldo)
}

/// Sincroniza el libro con un repositorio Git propio del usuario.
fn sincronizar_git_libro_impl(
    configuracion: ConfigDestinoSyncDTO,
    frase_de_paso: &str,
    ruta_clon: &Path,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
    reloj: &dyn Reloj,
) -> Result<ResultadoSyncDTO, String> {
    if configuracion.tipo != "git" {
        return Err("El tipo de destino debe ser \"git\".".to_string());
    }
    if configuracion.ruta_o_url.trim().is_empty() {
        return Err("La URL del repositorio no puede estar vacía.".to_string());
    }
    if configuracion.credenciales.is_some() {
        return Err(
            "Las credenciales se gestionan con el helper de Git; no se envían en la configuración."
                .to_string(),
        );
    }

    let transporte = TransporteGit::new(configuracion.ruta_o_url.clone(), ruta_clon);
    let reporte = aplicacion::casos_uso::sincronizar_libro::ejecutar(
        &transporte,
        frase_de_paso,
        repo_eventos,
        serializador,
        cifrador,
        reloj,
    )?;

    Ok(mapear_sincronizacion(&reporte))
}

#[tauri::command]
pub fn sincronizar_git_libro(
    configuracion: ConfigDestinoSyncDTO,
    frase_de_paso: String,
    repo_eventos: State<'_, Box<dyn RepositorioEventos>>,
    serializador: State<'_, Box<dyn SerializadorLibro>>,
    cifrador: State<'_, Box<dyn CifradorRespaldo>>,
    reloj: State<'_, Box<dyn Reloj>>,
    app: AppHandle,
) -> Result<ResultadoSyncDTO, ErrorRespaldoDTO> {
    const ERROR_REPOSITORIO: &str = "No se pudo llegar al repositorio; se reintentará.";
    let directorio = app
        .path()
        .app_data_dir()
        .map_err(|_| error_respaldo(ERROR_REPOSITORIO.to_string()))?;
    std::fs::create_dir_all(&directorio).map_err(|_| error_respaldo(ERROR_REPOSITORIO.to_string()))?;

    sincronizar_git_libro_impl(
        configuracion,
        &frase_de_paso,
        &directorio.join("git"),
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    ).map_err(error_respaldo)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn errores_serializan_codigos_sin_trazas() {
        for (entrada, esperado) in [
            ("Frase de paso incorrecta o archivo dañado.", "FraseIncorrectaOArchivoDanado"),
            ("Versión de formato no soportada: 2.0.", "VersionFormatoNoSoportada"),
            ("No se pudo llegar a la carpeta; se reintentará.", "DestinoInaccesibleOSinConexion"),
            ("Las credenciales fueron rechazadas; cómo actualizarlas.", "CredencialesGitInvalidas"),
            ("Evento con identificador repetido y contenido distinto", "EventoConflictoContenido"),
            ("panic src/secreto.rs", "ErrorOperacion"),
        ] {
            assert_eq!(serde_json::to_value(error_respaldo(entrada.to_string())).unwrap(), serde_json::json!({"codigo": esperado}));
        }
    }

    fn rechazado() -> EventoRechazado {
        EventoRechazado {
            id: "evt_conflictivo".to_string(),
            motivo: "Contenido distinto para el mismo ID".to_string(),
        }
    }

    #[test]
    fn la_importacion_informa_nuevos_existentes_y_rechazados_con_motivo() {
        let resumen = ResumenFusion {
            total_recibidos: 4,
            insertados: 2,
            ignorados_duplicados: 1,
            rechazados: vec![rechazado()],
        };

        let informe = mapear_importacion(&resumen, "2026-11-26T12:00:00Z".to_string());

        assert_eq!(informe.eventos_nuevos, 2);
        assert_eq!(informe.eventos_ya_existentes, 1);
        assert_eq!(informe.eventos_rechazados.len(), 1);
        assert_eq!(informe.eventos_rechazados[0].id_evento, "evt_conflictivo");
        assert_eq!(
            informe.eventos_rechazados[0].motivo,
            "Contenido distinto para el mismo ID"
        );
        assert_eq!(informe.fecha_ultima_sincronizacion, "2026-11-26T12:00:00Z");
    }

    #[test]
    fn la_sincronizacion_informa_enviados_recibidos_y_fecha() {
        let reporte = ReporteSincronizacionLibro {
            eventos_enviados: 3,
            eventos_recibidos: 2,
            eventos_ya_existentes: 40,
            rechazados: vec![rechazado()],
            fecha_ultima_sincronizacion: "2026-11-26T13:00:00Z".to_string(),
        };

        let informe = mapear_sincronizacion(&reporte);

        assert!(informe.exitoso);
        assert_eq!(informe.eventos_enviados, 3);
        assert_eq!(informe.eventos_recibidos, 2);
        assert_eq!(informe.eventos_rechazados.len(), 1);
        assert_eq!(informe.fecha_ultima_sync, "2026-11-26T13:00:00Z");
    }

    use std::path::Path;

    use infraestructura::{
        CifradorRespaldoAesGcm, RepositorioEventosSqlite, SerializadorLibroPlano, bd,
    };

    struct RelojFijo;

    impl Reloj for RelojFijo {
        fn ahora(&self) -> String {
            "2026-11-26T12:00:00Z".to_string()
        }
    }

    struct DirTemporal(PathBuf);

    impl DirTemporal {
        fn nueva() -> Self {
            let ruta = std::env::temp_dir().join(format!("cc-respaldo-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&ruta).unwrap();
            Self(ruta)
        }

        fn ruta(&self, nombre: &str) -> PathBuf {
            self.0.join(nombre)
        }

        fn como_texto(&self, nombre: &str) -> String {
            self.ruta(nombre).to_string_lossy().into_owned()
        }
    }

    impl Drop for DirTemporal {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn repositorio_vacio(ruta: &Path) -> RepositorioEventosSqlite {
        RepositorioEventosSqlite::new(bd::abrir(ruta).unwrap())
    }

    fn repositorio_con_un_evento(ruta: &Path) -> RepositorioEventosSqlite {
        let conexion = bd::abrir(ruta).unwrap();
        bd::insertar_evento_con_id(
            &conexion,
            "evt-cuenta-1",
            "CuentaCreada",
            "2026-11-20T10:00:00Z",
            "disp-a",
            &serde_json::json!({
                "id": "cuenta-1",
                "nombre": "Efectivo USD",
                "tipo": "Efectivo",
                "moneda_codigo": "USD",
                "saldo_inicial": "100.00",
            }),
        )
        .unwrap();
        RepositorioEventosSqlite::new(conexion)
    }

    #[test]
    fn exportar_plano_e_importar_en_otro_libro_restaura_el_evento() {
        let dir = DirTemporal::nueva();
        let origen = repositorio_con_un_evento(&dir.ruta("origen.db"));
        let serializador = SerializadorLibroPlano::new();
        let cifrador = CifradorRespaldoAesGcm::new();
        let reloj = RelojFijo;
        let ruta_respaldo = dir.como_texto("libro.txt");

        assert_eq!(
            exportar_libro_impl(
                ExportarLibroRequest {
                    ruta_destino: String::new(),
                    frase_de_paso: None,
                    cifrar: false,
                },
                &origen,
                &serializador,
                &cifrador,
                &reloj,
            )
            .unwrap_err(),
            "La ruta de destino no puede estar vacía."
        );
        assert_eq!(
            exportar_libro_impl(
                ExportarLibroRequest {
                    ruta_destino: ruta_respaldo.clone(),
                    frase_de_paso: None,
                    cifrar: true,
                },
                &origen,
                &serializador,
                &cifrador,
                &reloj,
            )
            .unwrap_err(),
            "La frase de paso es obligatoria para exportar cifrado."
        );

        let exportado = exportar_libro_impl(
            ExportarLibroRequest {
                ruta_destino: ruta_respaldo.clone(),
                frase_de_paso: None,
                cifrar: false,
            },
            &origen,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert!(exportado.exitoso);
        assert_eq!(exportado.cantidad_eventos, 1);
        assert_eq!(exportado.ruta_archivo, ruta_respaldo);

        let destino = repositorio_vacio(&dir.ruta("destino.db"));
        let importado = importar_libro_impl(
            ImportarLibroRequest {
                ruta_archivo: ruta_respaldo,
                frase_de_paso: String::new(),
            },
            &destino,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert!(importado.exitoso);
        assert_eq!(importado.reporte.eventos_nuevos, 1);
        assert_eq!(importado.reporte.eventos_ya_existentes, 0);
        assert_eq!(destino.listar_eventos_ordenados().unwrap().len(), 1);
    }

    #[test]
    fn exportar_cifrado_rechaza_frase_incorrecta_y_no_toca_el_libro() {
        let dir = DirTemporal::nueva();
        let origen = repositorio_con_un_evento(&dir.ruta("origen.db"));
        let serializador = SerializadorLibroPlano::new();
        let cifrador = CifradorRespaldoAesGcm::new();
        let reloj = RelojFijo;
        let ruta_respaldo = dir.como_texto("libro.ccbak");

        exportar_libro_impl(
            ExportarLibroRequest {
                ruta_destino: ruta_respaldo.clone(),
                frase_de_paso: Some("frase-segura-123".to_string()),
                cifrar: true,
            },
            &origen,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();

        let contenido = std::fs::read_to_string(&ruta_respaldo).unwrap();
        assert!(contenido.starts_with(aplicacion::puertos::MAGIA_RESPALDO_CIFRADO));
        assert!(!contenido.contains("cuenta-1"));

        let destino = repositorio_vacio(&dir.ruta("destino.db"));
        let error = importar_libro_impl(
            ImportarLibroRequest {
                ruta_archivo: ruta_respaldo.clone(),
                frase_de_paso: "frase-incorrecta-123".to_string(),
            },
            &destino,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap_err();
        assert_eq!(error, "Frase de paso incorrecta o archivo dañado.");
        assert!(destino.listar_eventos_ordenados().unwrap().is_empty());

        let importado = importar_libro_impl(
            ImportarLibroRequest {
                ruta_archivo: ruta_respaldo,
                frase_de_paso: "frase-segura-123".to_string(),
            },
            &destino,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(importado.reporte.eventos_nuevos, 1);
        assert_eq!(destino.listar_eventos_ordenados().unwrap().len(), 1);
    }

    #[test]
    fn sincronizar_carpeta_entre_dos_libros_converge_y_es_idempotente() {
        let dir = DirTemporal::nueva();
        let carpeta = dir.como_texto("compartida");
        let origen = repositorio_con_un_evento(&dir.ruta("origen.db"));
        let destino = repositorio_vacio(&dir.ruta("destino.db"));
        let serializador = SerializadorLibroPlano::new();
        let cifrador = CifradorRespaldoAesGcm::new();
        let reloj = RelojFijo;
        let frase = "frase-sincronizacion-123".to_string();

        assert_eq!(
            sincronizar_carpeta_libro_impl(
                String::new(),
                frase.clone(),
                &origen,
                &serializador,
                &cifrador,
                &reloj,
            )
            .unwrap_err(),
            "La ruta de la carpeta compartida no puede estar vacía."
        );

        let primero = sincronizar_carpeta_libro_impl(
            carpeta.clone(),
            frase.clone(),
            &origen,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert!(primero.exitoso);
        assert_eq!(primero.eventos_enviados, 1);
        assert_eq!(primero.eventos_recibidos, 0);

        let segundo = sincronizar_carpeta_libro_impl(
            carpeta.clone(),
            frase.clone(),
            &destino,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(segundo.eventos_recibidos, 1);
        // El evento recibido ya está en el destino: no se publica otra vez.
        assert_eq!(segundo.eventos_enviados, 0);

        let tercero = sincronizar_carpeta_libro_impl(
            carpeta.clone(),
            frase.clone(),
            &origen,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(tercero.eventos_enviados, 0);
        assert_eq!(tercero.eventos_recibidos, 0);

        let cuarto = sincronizar_carpeta_libro_impl(
            carpeta,
            frase,
            &destino,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(cuarto.eventos_enviados, 0);
        assert_eq!(cuarto.eventos_recibidos, 0);

        assert_eq!(origen.listar_eventos_ordenados().unwrap().len(), 1);
        assert_eq!(destino.listar_eventos_ordenados().unwrap().len(), 1);
    }

    fn git_en(directorio: &Path, argumentos: &[&str]) {
        let salida = std::process::Command::new("git")
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
    }

    fn clon_configurado(dir: &DirTemporal, remoto: &str, nombre: &str) -> PathBuf {
        git_en(&dir.0, &["clone", "--quiet", remoto, nombre]);
        let ruta = dir.ruta(nombre);
        git_en(
            &ruta,
            &["config", "user.email", "pruebas@cuentasclaras.local"],
        );
        git_en(&ruta, &["config", "user.name", "Prueba"]);
        ruta
    }

    #[test]
    fn sincronizar_git_entre_dos_libros_converge_y_es_idempotente() {
        let dir = DirTemporal::nueva();
        let remoto = dir.ruta("remoto.git");
        git_en(
            &dir.0,
            &["init", "--bare", "--quiet", remoto.to_str().unwrap()],
        );
        let remoto_texto = remoto.to_string_lossy().into_owned();

        let clon_a = clon_configurado(&dir, &remoto_texto, "clon-a");
        let clon_b = clon_configurado(&dir, &remoto_texto, "clon-b");

        let origen = repositorio_con_un_evento(&dir.ruta("origen.db"));
        let destino = repositorio_vacio(&dir.ruta("destino.db"));
        let serializador = SerializadorLibroPlano::new();
        let cifrador = CifradorRespaldoAesGcm::new();
        let reloj = RelojFijo;
        let frase = "frase-sincronizacion-git-123";
        let configuracion = ConfigDestinoSyncDTO {
            tipo: "git".to_string(),
            ruta_o_url: remoto_texto,
            credenciales: None,
        };

        assert_eq!(
            sincronizar_git_libro_impl(
                ConfigDestinoSyncDTO {
                    tipo: "carpeta".to_string(),
                    ..configuracion.clone()
                },
                frase,
                &clon_a,
                &origen,
                &serializador,
                &cifrador,
                &reloj,
            )
            .unwrap_err(),
            "El tipo de destino debe ser \"git\"."
        );
        assert_eq!(
            sincronizar_git_libro_impl(
                ConfigDestinoSyncDTO {
                    ruta_o_url: String::new(),
                    ..configuracion.clone()
                },
                frase,
                &clon_a,
                &origen,
                &serializador,
                &cifrador,
                &reloj,
            )
            .unwrap_err(),
            "La URL del repositorio no puede estar vacía."
        );
        assert_eq!(
            sincronizar_git_libro_impl(
                ConfigDestinoSyncDTO {
                    credenciales: Some("usuario:clave".to_string()),
                    ..configuracion.clone()
                },
                frase,
                &clon_a,
                &origen,
                &serializador,
                &cifrador,
                &reloj,
            )
            .unwrap_err(),
            "Las credenciales se gestionan con el helper de Git; no se envían en la configuración."
        );

        let primero = sincronizar_git_libro_impl(
            configuracion.clone(),
            frase,
            &clon_a,
            &origen,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert!(primero.exitoso);
        assert_eq!(primero.eventos_enviados, 1);
        assert_eq!(primero.eventos_recibidos, 0);

        let segundo = sincronizar_git_libro_impl(
            configuracion.clone(),
            frase,
            &clon_b,
            &destino,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(segundo.eventos_recibidos, 1);
        assert_eq!(segundo.eventos_enviados, 0);

        let tercero = sincronizar_git_libro_impl(
            configuracion.clone(),
            frase,
            &clon_a,
            &origen,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(tercero.eventos_enviados, 0);
        assert_eq!(tercero.eventos_recibidos, 0);

        let cuarto = sincronizar_git_libro_impl(
            configuracion,
            frase,
            &clon_b,
            &destino,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(cuarto.eventos_enviados, 0);
        assert_eq!(cuarto.eventos_recibidos, 0);

        assert_eq!(
            origen.listar_eventos_ordenados().unwrap(),
            destino.listar_eventos_ordenados().unwrap()
        );
    }
}
