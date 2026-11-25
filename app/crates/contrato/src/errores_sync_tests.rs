use crate::errores_sync::ErrorSincronizacion;

/// prueba RNF-18: los 5 errores tipados devuelven mensajes claros sin trazas técnicas
/// valida que el frontend recibirá texto limpio para mostrar al usuario
#[test]
fn errores_tipados_exponen_mensajes_amigables_sin_trazas() {
    let casos_de_prueba = vec![
        (
            ErrorSincronizacion::FraseIncorrectaOArchivoDanado,
            "Frase de paso incorrecta o archivo dañado.",
        ),
        (
            ErrorSincronizacion::VersionFormatoNoSoportada {
                version_archivo: "2.0".into(),
                version_soportada: "1.0".into(),
            },
            "Versión de formato no soportada. El archivo es v2.0 y la app soporta hasta v1.0.",
        ),
        (
            ErrorSincronizacion::DestinoInaccesibleOSinConexion,
            "No se pudo llegar al repositorio o a la carpeta; se reintentará.",
        ),
        (
            ErrorSincronizacion::CredencialesGitInvalidas,
            "Las credenciales fueron rechazadas; por favor actualízalas en configuración.",
        ),
        (
            ErrorSincronizacion::EventoConflictoContenido {
                id_evento: "evt_99".into(),
            },
            "Evento con identificador repetido y contenido distinto (ID: evt_99).",
        ),
    ];

    for (error, mensaje_esperado) in casos_de_prueba {
        let mensaje_real = error.to_string();

        // verificamos que el mensaje coincide exactamente con la especificacion
        assert_eq!(
            mensaje_real, mensaje_esperado,
            "el mensaje del error no cumple la especificacion"
        );

        // garantizamos que no se filtran trazas tecnicas al frontend
        assert!(
            !mensaje_real.contains("panicked"),
            "el error expone trazas de panic"
        );
        assert!(
            !mensaje_real.contains("src/"),
            "el error expone rutas de codigo"
        );
        assert!(
            !mensaje_real.contains("unwrap"),
            "el error expone metodos de rust"
        );
    }
}
