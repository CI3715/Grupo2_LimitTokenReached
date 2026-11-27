use std::cmp::Ordering;
use std::collections::HashMap;

/// Representa un evento inmutable del libro personal (INV-13, INV-14).
/// `datos_canonicos` contiene el payload JSON serializado de forma canónica
/// (claves ordenadas y sin espacios superfluos) para comparar contenido con exactitud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoLibro {
    pub id: String,
    pub tipo: String,
    pub registrado_en: String,
    pub dispositivo: String,
    pub datos_canonicos: String,
}

impl EventoLibro {
    /// Compara si dos eventos tienen exactamente el mismo contenido y metadatos.
    pub fn mismo_contenido(&self, otro: &Self) -> bool {
        self.id == otro.id
            && self.tipo == otro.tipo
            && self.registrado_en == otro.registrado_en
            && self.dispositivo == otro.dispositivo
            && self.datos_canonicos == otro.datos_canonicos
    }

    /// Explica qué campo difiere cuando dos eventos comparten el mismo `id`.
    pub fn motivo_conflicto(&self, entrante: &Self) -> String {
        if self.tipo != entrante.tipo {
            return format!(
                "Conflicto en evento {}: tipo local '{}' difiere del entrante '{}'",
                self.id, self.tipo, entrante.tipo
            );
        }
        if self.registrado_en != entrante.registrado_en {
            return format!(
                "Conflicto en evento {}: marca temporal local '{}' difiere de '{}'",
                self.id, self.registrado_en, entrante.registrado_en
            );
        }
        if self.dispositivo != entrante.dispositivo {
            return format!(
                "Conflicto en evento {}: dispositivo origen local '{}' difiere de '{}'",
                self.id, self.dispositivo, entrante.dispositivo
            );
        }
        format!(
            "Conflicto en evento {}: el contenido (datos) difiere del evento ya registrado",
            self.id
        )
    }
}

/// Orden total determinista exigido por la especificación:
/// (timestamp / registrado_en, device_id / dispositivo, event_id / id).
impl Ord for EventoLibro {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.registrado_en.as_str(),
            self.dispositivo.as_str(),
            self.id.as_str(),
        )
            .cmp(&(
                other.registrado_en.as_str(),
                other.dispositivo.as_str(),
                other.id.as_str(),
            ))
    }
}

impl PartialOrd for EventoLibro {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Detalle de un evento rechazado durante la fusión sin abortar el resto del lote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoRechazado {
    pub id: String,
    pub motivo: String,
}

/// Resultado consolidado de una operación de fusión de eventos.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResumenFusion {
    pub total_recibidos: usize,
    pub insertados: usize,
    pub ignorados_duplicados: usize,
    pub rechazados: Vec<EventoRechazado>,
}

/// Ordena en sitio una colección de eventos según `(registrado_en, dispositivo, id)`.
pub fn ordenar_eventos_determinista(eventos: &mut [EventoLibro]) {
    eventos.sort();
}

/// Evalúa un lote de eventos entrantes contra los eventos ya existentes en el libro:
/// - Ignora eventos idénticos (idempotencia).
/// - Rechaza con motivo explícito eventos con el mismo `id` y distinto contenido, sin abortar el lote.
/// - Devuelve los nuevos eventos válidos a insertar (ordenados) y el `ResumenFusion`.
pub fn evaluar_fusion_lote(
    existentes: &HashMap<String, EventoLibro>,
    entrantes: &[EventoLibro],
) -> (Vec<EventoLibro>, ResumenFusion) {
    let mut resumen = ResumenFusion {
        total_recibidos: entrantes.len(),
        ..Default::default()
    };

    let mut conocidos: HashMap<&str, &EventoLibro> =
        existentes.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let mut nuevos = Vec::new();

    for evento in entrantes {
        if let Some(previo) = conocidos.get(evento.id.as_str()) {
            if previo.mismo_contenido(evento) {
                resumen.ignorados_duplicados += 1;
            } else {
                resumen.rechazados.push(EventoRechazado {
                    id: evento.id.clone(),
                    motivo: previo.motivo_conflicto(evento),
                });
            }
        } else {
            conocidos.insert(evento.id.as_str(), evento);
            nuevos.push(evento.clone());
            resumen.insertados += 1;
        }
    }

    ordenar_eventos_determinista(&mut nuevos);
    (nuevos, resumen)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn evento(id: &str, ts: &str, disp: &str, datos: &str) -> EventoLibro {
        EventoLibro {
            id: id.to_string(),
            tipo: "MovimientoRegistrado".to_string(),
            registrado_en: ts.to_string(),
            dispositivo: disp.to_string(),
            datos_canonicos: datos.to_string(),
        }
    }

    #[test]
    fn ordena_por_timestamp_luego_dispositivo_luego_id() {
        let e1 = evento("e2", "2026-11-26T10:00:00Z", "disp-b", "{}");
        let e2 = evento("e1", "2026-11-26T10:00:00Z", "disp-b", "{}");
        let e3 = evento("e9", "2026-11-26T10:00:00Z", "disp-a", "{}");
        let e4 = evento("e0", "2026-11-26T09:59:59Z", "disp-z", "{}");

        let mut lista = vec![e1.clone(), e2.clone(), e3.clone(), e4.clone()];
        ordenar_eventos_determinista(&mut lista);

        assert_eq!(lista, vec![e4, e3, e2, e1]);
    }

    #[test]
    fn fusion_ignora_identicos_y_rechaza_conflictos_sin_abortar_lote() {
        let existente = evento("e1", "2026-11-26T10:00:00Z", "disp-a", r#"{"monto":"10"}"#);
        let mut mapa = HashMap::new();
        mapa.insert(existente.id.clone(), existente.clone());

        let duplicado = existente;
        let conflicto = evento("e1", "2026-11-26T10:00:00Z", "disp-a", r#"{"monto":"99"}"#);
        let nuevo = evento("e2", "2026-11-26T10:05:00Z", "disp-b", r#"{"monto":"25"}"#);

        let (para_insertar, resumen) =
            evaluar_fusion_lote(&mapa, &[duplicado, conflicto, nuevo.clone()]);

        assert_eq!(resumen.total_recibidos, 3);
        assert_eq!(resumen.ignorados_duplicados, 1);
        assert_eq!(resumen.rechazados.len(), 1);
        assert_eq!(resumen.rechazados[0].id, "e1");
        assert_eq!(resumen.insertados, 1);
        assert_eq!(para_insertar, vec![nuevo]);
    }

    #[test]
    fn distinta_marca_temporal_o_dispositivo_se_rechaza_con_motivo() {
        let local = evento("e1", "2026-11-26T10:00:00Z", "disp-a", "{}");
        let otra_fecha = evento("e1", "2026-11-26T11:00:00Z", "disp-a", "{}");
        let otro_dispositivo = evento("e1", "2026-11-26T10:00:00Z", "disp-b", "{}");

        assert!(!local.mismo_contenido(&otra_fecha));
        assert!(!local.mismo_contenido(&otro_dispositivo));
        assert!(
            local
                .motivo_conflicto(&otra_fecha)
                .contains("marca temporal")
        );
        assert!(
            local
                .motivo_conflicto(&otro_dispositivo)
                .contains("dispositivo")
        );
    }
}
