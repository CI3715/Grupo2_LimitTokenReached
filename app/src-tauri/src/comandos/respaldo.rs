//! Comandos Tauri del respaldo cifrado y de la sincronización por carpeta compartida.
//!
//! Solo exponen los casos de uso de la Entrega 8. La interfaz (doble entrada de la
//! frase, advertencia de irrecuperabilidad y botón “Sincronizar ahora”) queda fuera
//! de este crate.

use std::path::PathBuf;

use aplicacion::puertos::{
    CifradorRespaldo, Reloj, ReporteSincronizacionLibro, RepositorioEventos, SerializadorLibro,
};
use contrato::respaldo_sincronizacion::{EventoRechazadoDTO, ReporteSincronizacionDTO};
use contrato::{
    ExportarLibroRequest, ExportarLibroResponse, ImportarLibroRequest, ImportarLibroResponse,
    ResultadoSyncDTO,
};
use dominio::libro::{EventoRechazado, ResumenFusion};
use tauri::State;

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
) -> Result<ExportarLibroResponse, String> {
    exportar_libro_impl(
        datos,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    )
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
) -> Result<ImportarLibroResponse, String> {
    importar_libro_impl(
        datos,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    )
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

    let reporte = aplicacion::casos_uso::sincronizar_carpeta_libro::ejecutar(
        &PathBuf::from(&ruta_carpeta),
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
) -> Result<ResultadoSyncDTO, String> {
    sincronizar_carpeta_libro_impl(
        ruta_carpeta,
        frase_de_paso,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;

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
}
