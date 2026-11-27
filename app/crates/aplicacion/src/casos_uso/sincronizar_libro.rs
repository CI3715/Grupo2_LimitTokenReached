//! Sincronización del libro con cualquier destino que implemente `TransporteSincronizacion`.

use crate::puertos::{
    CifradorRespaldo, Reloj, ReporteSincronizacionLibro, RepositorioEventos, SerializadorLibro,
    TransporteSincronizacion,
};

/// Trae los lotes del destino, fusiona los eventos recibidos y publica un lote
/// nuevo solo con los eventos locales que el destino todavía no tenía.
pub fn ejecutar(
    transporte: &dyn TransporteSincronizacion,
    frase_paso: &str,
    repo_eventos: &dyn RepositorioEventos,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorRespaldo,
    reloj: &dyn Reloj,
) -> Result<ReporteSincronizacionLibro, String> {
    let device_id = repo_eventos.obtener_device_id()?;
    let lotes = transporte.leer_lotes()?;

    let mut eventos_entrantes = Vec::new();
    let mut ya_enviados = std::collections::HashSet::new();

    for lote in lotes {
        let plano = cifrador.descifrar(&lote.contenido, frase_paso)?;
        let libro = serializador.deserializar_plano(&plano)?;

        // Cualquier evento presente en el destino cuenta como ya publicado,
        // lo haya subido este dispositivo o cualquier otro. Mirar solo los lotes
        // propios haría que el dispositivo reenviase lo que acaba de recibir.
        ya_enviados.extend(libro.eventos.iter().map(|evento| evento.id.clone()));
        eventos_entrantes.extend(libro.eventos);
    }

    let resumen = repo_eventos.fusionar_lote_transaccional(&eventos_entrantes)?;
    let eventos_locales = repo_eventos.listar_eventos_ordenados()?;
    let para_enviar: Vec<_> = eventos_locales
        .into_iter()
        .filter(|evento| !ya_enviados.contains(&evento.id))
        .collect();
    let eventos_enviados = para_enviar.len();

    if !para_enviar.is_empty() {
        let plano = serializador.serializar_plano(&device_id, &reloj.ahora(), &para_enviar)?;
        let cifrado = cifrador.cifrar(&plano, frase_paso)?;
        transporte.publicar_lote(&device_id, &cifrado)?;
    }

    Ok(ReporteSincronizacionLibro {
        eventos_enviados,
        eventos_recibidos: resumen.insertados,
        eventos_ya_existentes: resumen.ignorados_duplicados,
        rechazados: resumen.rechazados,
        fecha_ultima_sincronizacion: reloj.ahora(),
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::puertos::{LibroDeserializado, LoteRecibido, ReporteIntegridad};
    use dominio::libro::{EventoLibro, EventoRechazado, ResumenFusion};
    use std::sync::Mutex;

    const FECHA: &str = "2026-11-26T12:00:00Z";

    struct RelojFijo;

    impl Reloj for RelojFijo {
        fn ahora(&self) -> String {
            FECHA.to_string()
        }
    }

    struct TransporteMemoria {
        lotes: Mutex<Vec<LoteRecibido>>,
    }

    impl TransporteMemoria {
        fn nueva() -> Self {
            Self {
                lotes: Mutex::new(Vec::new()),
            }
        }

        fn cantidad(&self) -> usize {
            self.lotes.lock().unwrap().len()
        }
    }

    impl TransporteSincronizacion for TransporteMemoria {
        fn leer_lotes(&self) -> Result<Vec<LoteRecibido>, String> {
            Ok(self.lotes.lock().unwrap().clone())
        }

        fn publicar_lote(&self, dispositivo: &str, contenido: &[u8]) -> Result<(), String> {
            self.lotes.lock().unwrap().push(LoteRecibido {
                dispositivo: dispositivo.to_string(),
                contenido: contenido.to_vec(),
            });
            Ok(())
        }
    }

    struct SerializadorFalso;

    impl SerializadorLibro for SerializadorFalso {
        fn serializar_plano(
            &self,
            device_id: &str,
            exportado_en: &str,
            eventos: &[EventoLibro],
        ) -> Result<Vec<u8>, String> {
            let mut lineas = vec![format!("PRUEBA\t{device_id}\t{exportado_en}")];
            for evento in eventos {
                lineas.push(
                    [
                        evento.id.as_str(),
                        evento.tipo.as_str(),
                        evento.registrado_en.as_str(),
                        evento.dispositivo.as_str(),
                        evento.datos_canonicos.as_str(),
                    ]
                    .join("\t"),
                );
            }
            Ok(lineas.join("\n").into_bytes())
        }

        fn deserializar_plano(&self, contenido: &[u8]) -> Result<LibroDeserializado, String> {
            let texto = std::str::from_utf8(contenido)
                .map_err(|_| "Contenido inválido en el lote de prueba.".to_string())?;
            let mut lineas = texto.lines();
            let cabecera = lineas
                .next()
                .ok_or_else(|| "Lote de prueba sin cabecera.".to_string())?;
            let partes: Vec<_> = cabecera.split('\t').collect();
            if partes.len() != 3 || partes[0] != "PRUEBA" {
                return Err("Cabecera de lote de prueba no reconocida.".to_string());
            }

            let mut eventos = Vec::new();
            for linea in lineas {
                let partes: Vec<_> = linea.split('\t').collect();
                if partes.len() != 5 {
                    return Err("Evento de lote de prueba mal formado.".to_string());
                }
                eventos.push(EventoLibro {
                    id: partes[0].to_string(),
                    tipo: partes[1].to_string(),
                    registrado_en: partes[2].to_string(),
                    dispositivo: partes[3].to_string(),
                    datos_canonicos: partes[4].to_string(),
                });
            }

            Ok(LibroDeserializado {
                version: partes[0].to_string(),
                exportado_en: partes[2].to_string(),
                eventos,
            })
        }
    }

    struct CifradorFalso;

    impl CifradorRespaldo for CifradorFalso {
        fn cifrar(&self, datos_planos: &[u8], _frase_paso: &str) -> Result<Vec<u8>, String> {
            Ok(datos_planos.to_vec())
        }

        fn descifrar(&self, paquete_cifrado: &[u8], _frase_paso: &str) -> Result<Vec<u8>, String> {
            Ok(paquete_cifrado.to_vec())
        }
    }

    struct RepositorioFalso {
        device_id: String,
        eventos: Mutex<Vec<EventoLibro>>,
    }

    impl RepositorioFalso {
        fn nuevo(device_id: &str, eventos: Vec<EventoLibro>) -> Self {
            Self {
                device_id: device_id.to_string(),
                eventos: Mutex::new(eventos),
            }
        }
    }

    impl RepositorioEventos for RepositorioFalso {
        fn obtener_device_id(&self) -> Result<String, String> {
            Ok(self.device_id.clone())
        }

        fn listar_eventos_ordenados(&self) -> Result<Vec<EventoLibro>, String> {
            let mut eventos = self.eventos.lock().unwrap().clone();
            eventos.sort();
            Ok(eventos)
        }

        fn fusionar_lote_transaccional(
            &self,
            eventos: &[EventoLibro],
        ) -> Result<ResumenFusion, String> {
            let mut guardados = self.eventos.lock().unwrap();
            let mut resumen = ResumenFusion {
                total_recibidos: eventos.len(),
                ..ResumenFusion::default()
            };

            for entrante in eventos {
                match guardados.iter().find(|evento| evento.id == entrante.id) {
                    Some(existente) if existente.mismo_contenido(entrante) => {
                        resumen.ignorados_duplicados += 1;
                    }
                    Some(existente) => resumen.rechazados.push(EventoRechazado {
                        id: entrante.id.clone(),
                        motivo: existente.motivo_conflicto(entrante),
                    }),
                    None => {
                        guardados.push(entrante.clone());
                        resumen.insertados += 1;
                    }
                }
            }

            Ok(resumen)
        }

        fn reconstruir_proyecciones_desde_eventos(&self) -> Result<(), String> {
            Ok(())
        }

        fn verificar_integridad(&self) -> Result<ReporteIntegridad, String> {
            Ok(ReporteIntegridad {
                integro: true,
                total_eventos_verificados: self.eventos.lock().unwrap().len(),
                cuentas_verificadas: 0,
                discrepancias: Vec::new(),
            })
        }
    }

    fn evento(id: &str, dispositivo: &str) -> EventoLibro {
        EventoLibro {
            id: id.to_string(),
            tipo: "CuentaCreada".to_string(),
            registrado_en: "2026-11-20T10:00:00Z".to_string(),
            dispositivo: dispositivo.to_string(),
            datos_canonicos: format!("{{\"id\":\"{id}\"}}"),
        }
    }

    #[test]
    fn el_caso_de_uso_converge_y_es_idempotente_con_cualquier_transporte() {
        let transporte = TransporteMemoria::nueva();
        let serializador = SerializadorFalso;
        let cifrador = CifradorFalso;
        let reloj = RelojFijo;

        let laptop = RepositorioFalso::nuevo("disp-a", vec![evento("evt-1", "disp-a")]);
        let primero = ejecutar(
            &transporte,
            "frase",
            &laptop,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(primero.eventos_enviados, 1);
        assert_eq!(primero.eventos_recibidos, 0);
        assert_eq!(primero.fecha_ultima_sincronizacion, FECHA);
        assert_eq!(transporte.cantidad(), 1);

        let telefono = RepositorioFalso::nuevo("disp-b", Vec::new());
        let segundo = ejecutar(
            &transporte,
            "frase",
            &telefono,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(segundo.eventos_enviados, 0);
        assert_eq!(segundo.eventos_recibidos, 1);
        // El evento recibido ya está en el destino: no se publica otra vez.
        assert_eq!(transporte.cantidad(), 1);

        let tercero = ejecutar(
            &transporte,
            "frase",
            &laptop,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(tercero.eventos_enviados, 0);
        assert_eq!(tercero.eventos_recibidos, 0);
        assert_eq!(transporte.cantidad(), 1);

        let cuarto = ejecutar(
            &transporte,
            "frase",
            &telefono,
            &serializador,
            &cifrador,
            &reloj,
        )
        .unwrap();
        assert_eq!(cuarto.eventos_enviados, 0);
        assert_eq!(cuarto.eventos_recibidos, 0);
        assert_eq!(transporte.cantidad(), 1);
        assert_eq!(
            laptop.listar_eventos_ordenados().unwrap(),
            telefono.listar_eventos_ordenados().unwrap()
        );
    }
}
