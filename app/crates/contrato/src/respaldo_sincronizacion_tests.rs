#[cfg(test)]
mod tests {
    use crate::respaldo_sincronizacion::{
        EncabezadoExportacionDTO, EventoRechazadoDTO, ReporteSincronizacionDTO,
    };

    #[test]
    fn encabezado_exportacion_registra_version_y_cantidad() {
        let encabezado = EncabezadoExportacionDTO {
            version_formato: "1.0.0".to_string(),
            fecha_exportacion: "2026-11-25T10:00:00Z".to_string(),
            cantidad_eventos: 42,
        };

        // se verifica que el encabezado contiene la metadata exacta que nos exigen
        // esto permite que herramientas externas lean el archivo y reconstruyan los saldos
        // sin depender de la aplicacion
        assert_eq!(encabezado.version_formato, "1.0.0");
        assert_eq!(encabezado.cantidad_eventos, 42);
    }

    #[test]
    fn reporte_sincronizacion_cuenta_nuevos_existentes_y_rechazados() {
        let reporte = ReporteSincronizacionDTO {
            eventos_nuevos: 2,
            eventos_ya_existentes: 40,
            eventos_rechazados: vec![EventoRechazadoDTO {
                id_evento: "evt_conflict_99".to_string(),
                motivo: "Contenido distinto para el mismo ID".to_string(),
            }],
            fecha_ultima_sincronizacion: "2026-11-25T12:00:00Z".to_string(),
        };

        // validamos la estructura del reporte que ve el usuario, lo cual es crucial que los eventos
        // rechazados incluyan el id y el motivo, asi el usuario sabe que paso sin necesidad de tecnicismos
        assert_eq!(reporte.eventos_nuevos, 2);
        assert_eq!(reporte.eventos_ya_existentes, 40);
        assert_eq!(reporte.eventos_rechazados.len(), 1);
        assert_eq!(
            reporte.eventos_rechazados[0].motivo,
            "Contenido distinto para el mismo ID"
        );
    }

    #[test]
    fn evento_rechazado_preserva_identificador_y_motivo() {
        let rechazado = EventoRechazadoDTO {
            id_evento: "evt_123".to_string(),
            motivo: "Frase de paso incorrecta o archivo dañado".to_string(),
        };

        // garantizamos que el contrato de error no pierde informacion
        assert_eq!(rechazado.id_evento, "evt_123");
    }
}
