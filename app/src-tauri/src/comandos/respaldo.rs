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
#[tauri::command]
pub fn exportar_libro(
    datos: ExportarLibroRequest,
    repo_eventos: State<'_, Box<dyn RepositorioEventos>>,
    serializador: State<'_, Box<dyn SerializadorLibro>>,
    cifrador: State<'_, Box<dyn CifradorRespaldo>>,
    reloj: State<'_, Box<dyn Reloj>>,
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
            repo_eventos.inner().as_ref(),
            serializador.inner().as_ref(),
            cifrador.inner().as_ref(),
            reloj.inner().as_ref(),
        )?
    } else {
        aplicacion::casos_uso::exportar_respaldo_cifrado::ejecutar_plano(
            &ruta,
            repo_eventos.inner().as_ref(),
            serializador.inner().as_ref(),
            reloj.inner().as_ref(),
        )?
    };

    Ok(ExportarLibroResponse {
        exitoso: true,
        ruta_archivo: datos.ruta_destino,
        cantidad_eventos: cantidad_eventos as u64,
    })
}

/// Importa un respaldo cifrado o plano. El caso de uso valida todo el archivo
/// antes de abrir la transacción de fusión, por lo que un archivo inválido no
/// modifica el libro.
#[tauri::command]
pub fn importar_libro(
    datos: ImportarLibroRequest,
    repo_eventos: State<'_, Box<dyn RepositorioEventos>>,
    serializador: State<'_, Box<dyn SerializadorLibro>>,
    cifrador: State<'_, Box<dyn CifradorRespaldo>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<ImportarLibroResponse, String> {
    if datos.ruta_archivo.trim().is_empty() {
        return Err("La ruta del archivo no puede estar vacía.".to_string());
    }

    let resumen = aplicacion::casos_uso::importar_respaldo_cifrado::ejecutar(
        &PathBuf::from(&datos.ruta_archivo),
        &datos.frase_de_paso,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
    )?;

    Ok(ImportarLibroResponse {
        exitoso: true,
        reporte: mapear_importacion(&resumen, reloj.inner().as_ref().ahora()),
    })
}

/// Sincroniza el libro con una carpeta compartida: trae los lotes `*.ccbak`,
/// fusiona los eventos recibidos y publica un lote nuevo e inmutable con los
/// eventos locales que aún no se habían enviado.
#[tauri::command]
pub fn sincronizar_carpeta_libro(
    ruta_carpeta: String,
    frase_de_paso: String,
    repo_eventos: State<'_, Box<dyn RepositorioEventos>>,
    serializador: State<'_, Box<dyn SerializadorLibro>>,
    cifrador: State<'_, Box<dyn CifradorRespaldo>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<ResultadoSyncDTO, String> {
    if ruta_carpeta.trim().is_empty() {
        return Err("La ruta de la carpeta compartida no puede estar vacía.".to_string());
    }

    let reporte = aplicacion::casos_uso::sincronizar_carpeta_libro::ejecutar(
        &PathBuf::from(&ruta_carpeta),
        &frase_de_paso,
        repo_eventos.inner().as_ref(),
        serializador.inner().as_ref(),
        cifrador.inner().as_ref(),
        reloj.inner().as_ref(),
    )?;

    Ok(mapear_sincronizacion(&reporte))
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
}
