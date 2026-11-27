//! Suite del motor de fusión de eventos (Entrega 8, RF-INT-02 a RF-INT-05, INV-13, INV-14).
//!
//! Corre contra SQLite real (archivos temporales) y el motor de fusión de la app:
//! idempotencia, conmutatividad del escenario 2.6, rechazo aislado de conflictos y
//! deduplicación de gastos de grupo (E7). Los datos salen de `tests/comun`.

mod comun;

use aplicacion::casos_uso::aplicar_eventos_grupo::{ResultadoPreparacion, preparar_evento};
use aplicacion::casos_uso::{
    exportar_respaldo_cifrado, importar_respaldo_cifrado, sincronizar_carpeta_libro,
};
use aplicacion::puertos::{
    AplicacionGastoPlan, CifradorRespaldo, LoteSincronizacionPlan, Reloj, RepositorioCuentas,
    RepositorioEventos, RepositorioSincronizacion,
};
use comun::*;
use contrato::{CuentaGrupoSeleccionDTO, EventoCambioDTO};
use dominio::libro::EventoLibro;
use infraestructura::{RelojSistema, RepositorioSincronizacionSqlite, SerializadorLibroPlano, bd};
use rust_decimal::Decimal;
use serde_json::json;
use std::path::Path;

/// Cambio que se le hace a un evento para provocar un conflicto.
type Modificacion = fn(&mut EventoPrueba);

fn d(texto: &str) -> Decimal {
    texto.parse().unwrap()
}

/// Laptop (base + sus 2 eventos) y teléfono (base + sus 2 eventos), cada uno con su base.
fn laptop_y_telefono(escenario: &Escenario26) -> (Dispositivo, Dispositivo) {
    let mut en_laptop = escenario.base.clone();
    en_laptop.extend(escenario.laptop.clone());
    let mut en_telefono = escenario.base.clone();
    en_telefono.extend(escenario.telefono.clone());
    (
        Dispositivo::con_eventos(&en_laptop),
        Dispositivo::con_eventos(&en_telefono),
    )
}

// ---------------------------------------------------------------------------------------
// Idempotencia: importar el mismo archivo dos veces es igual a importarlo una vez
// ---------------------------------------------------------------------------------------

#[test]
fn fusionar_el_mismo_lote_dos_veces_es_igual_a_fusionarlo_una_vez() {
    let escenario = Escenario26::nuevo();
    let dispositivo = Dispositivo::nuevo();

    let primera = dispositivo.fusionar(&escenario.base);
    assert_eq!(primera.total_recibidos, 40);
    assert_eq!(primera.insertados, 40);
    assert_eq!(primera.ignorados_duplicados, 0);
    assert!(primera.rechazados.is_empty());
    let despues_de_una = dispositivo.foto();

    let segunda = dispositivo.fusionar(&escenario.base);
    assert_eq!(segunda.total_recibidos, 40);
    assert_eq!(segunda.insertados, 0);
    assert_eq!(segunda.ignorados_duplicados, 40);
    assert!(segunda.rechazados.is_empty());

    assert_eq!(dispositivo.foto(), despues_de_una);
    assert_eq!(dispositivo.contar_eventos(), 40);
    assert_eq!(dispositivo.saldo(), d(SALDO_BASE));
}

#[test]
fn importar_el_mismo_archivo_dos_veces_es_igual_a_importarlo_una_vez() {
    let origen = Dispositivo::con_eventos(&Escenario26::nuevo().todos());
    let archivo = ArchivoTemporal::nuevo("respaldo", "txt");
    let serializador = SerializadorLibroPlano::new();

    let exportados = exportar_respaldo_cifrado::ejecutar_plano(
        &archivo.0,
        &origen.repo,
        &serializador,
        &RelojSistema,
    )
    .unwrap();
    assert_eq!(exportados, 44);

    // Instalación nueva: restaura todo el libro.
    let destino = Dispositivo::nuevo();
    let primera = importar_respaldo_cifrado::ejecutar(
        &archivo.0,
        "",
        &destino.repo,
        &serializador,
        &CifradorDePrueba,
    )
    .unwrap();
    assert_eq!(primera.insertados, 44);
    assert_eq!(primera.ignorados_duplicados, 0);
    assert!(primera.rechazados.is_empty());
    let despues_de_una = destino.foto();
    assert_eq!(
        despues_de_una,
        origen.foto(),
        "la restauración debe ser fiel"
    );
    assert_eq!(destino.saldo(), d(SALDO_CONVERGIDO));

    // Segunda importación del mismo archivo: no cambia nada.
    let segunda = importar_respaldo_cifrado::ejecutar(
        &archivo.0,
        "",
        &destino.repo,
        &serializador,
        &CifradorDePrueba,
    )
    .unwrap();
    assert_eq!(segunda.insertados, 0);
    assert_eq!(segunda.ignorados_duplicados, 44);
    assert!(segunda.rechazados.is_empty());
    assert_eq!(destino.foto(), despues_de_una);
}

// ---------------------------------------------------------------------------------------
// Conmutatividad del escenario 2.6: Laptop -> Teléfono y Teléfono -> Laptop convergen
// ---------------------------------------------------------------------------------------

#[test]
fn laptop_a_telefono_y_telefono_a_laptop_convergen_a_los_mismos_44_eventos() {
    let escenario = Escenario26::nuevo();

    // Orden 1: primero el teléfono recibe de la laptop, luego la laptop del teléfono.
    let (laptop, telefono) = laptop_y_telefono(&escenario);
    let al_telefono = telefono.fusionar_libro(&laptop.eventos());
    assert_eq!(al_telefono.insertados, 2);
    assert_eq!(al_telefono.ignorados_duplicados, 40);
    let a_la_laptop = laptop.fusionar_libro(&telefono.eventos());
    assert_eq!(a_la_laptop.insertados, 2);
    assert_eq!(a_la_laptop.ignorados_duplicados, 42);
    assert!(al_telefono.rechazados.is_empty() && a_la_laptop.rechazados.is_empty());

    for dispositivo in [&laptop, &telefono] {
        assert_eq!(dispositivo.contar_eventos(), EVENTOS_CONVERGIDOS);
        assert_eq!(dispositivo.saldo(), d(SALDO_CONVERGIDO));
        assert_eq!(dispositivo.saldo_recalculado(), d(SALDO_CONVERGIDO));
    }
    assert_eq!(laptop.foto(), telefono.foto());
    let foto_orden_1 = laptop.foto();

    // Orden 2: primero la laptop recibe del teléfono, luego el teléfono de la laptop.
    let (laptop, telefono) = laptop_y_telefono(&escenario);
    let a_la_laptop = laptop.fusionar_libro(&telefono.eventos());
    assert_eq!(a_la_laptop.insertados, 2);
    assert_eq!(a_la_laptop.ignorados_duplicados, 40);
    let al_telefono = telefono.fusionar_libro(&laptop.eventos());
    assert_eq!(al_telefono.insertados, 2);
    assert_eq!(al_telefono.ignorados_duplicados, 42);

    for dispositivo in [&laptop, &telefono] {
        assert_eq!(dispositivo.contar_eventos(), EVENTOS_CONVERGIDOS);
        assert_eq!(dispositivo.saldo(), d(SALDO_CONVERGIDO));
    }
    assert_eq!(laptop.foto(), telefono.foto());
    assert_eq!(
        laptop.foto(),
        foto_orden_1,
        "los dos órdenes deben dar el mismo libro"
    );
}

/// Prueba por propiedades: los 44 eventos llegan en orden aleatorio y troceados en lotes
/// de tamaño aleatorio; el libro final siempre es el mismo. (La cuenta va primero porque
/// los movimientos necesitan que exista su cuenta.)
#[test]
fn cualquier_orden_y_troceado_de_los_44_eventos_converge_al_mismo_libro() {
    let todos = Escenario26::nuevo().todos();
    let referencia = Dispositivo::con_eventos(&todos).foto();

    for semilla in 1..=20_u64 {
        let mut azar = Aleatorio::nuevo(semilla);
        let mut resto = todos[1..].to_vec();
        azar.mezclar(&mut resto);

        let dispositivo = Dispositivo::con_eventos(&todos[..1]);
        let mut pendiente = resto.as_slice();
        while !pendiente.is_empty() {
            let tamano = (1 + azar.hasta(9)).min(pendiente.len());
            let (lote, sobra) = pendiente.split_at(tamano);
            let resumen = dispositivo.fusionar(lote);
            assert!(
                resumen.rechazados.is_empty(),
                "semilla {semilla}: {:?}",
                resumen.rechazados
            );
            pendiente = sobra;
        }

        assert_eq!(dispositivo.foto(), referencia, "semilla {semilla}");
        assert_eq!(
            dispositivo.saldo(),
            d(SALDO_CONVERGIDO),
            "semilla {semilla}"
        );
        assert_eq!(
            dispositivo.saldo_recalculado(),
            d(SALDO_CONVERGIDO),
            "semilla {semilla}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Rechazo aislado: mismo ID con contenido modificado
// ---------------------------------------------------------------------------------------

#[test]
fn id_repetido_con_contenido_distinto_se_rechaza_y_el_resto_del_lote_entra() {
    let escenario = Escenario26::nuevo();
    let mut local = escenario.base.clone();
    local.extend(escenario.laptop.clone());
    let dispositivo = Dispositivo::con_eventos(&local);
    let antes = dispositivo.foto();

    // El primer evento de la laptop llega con otro monto; los del teléfono son nuevos y válidos.
    let mut alterado = escenario.laptop[0].clone();
    alterado.datos["monto"] = json!("999.00");
    let lote = vec![
        alterado,
        escenario.telefono[0].clone(),
        escenario.telefono[1].clone(),
    ];
    let resumen = dispositivo.fusionar(&lote);

    assert_eq!(resumen.total_recibidos, 3);
    assert_eq!(resumen.insertados, 2);
    assert_eq!(resumen.ignorados_duplicados, 0);
    assert_eq!(resumen.rechazados.len(), 1);
    assert_eq!(resumen.rechazados[0].id, escenario.laptop[0].id);
    assert!(
        resumen.rechazados[0]
            .motivo
            .contains(&escenario.laptop[0].id),
        "el motivo debe nombrar el evento: {}",
        resumen.rechazados[0].motivo
    );

    // El evento local original no se tocó y el monto alterado nunca se aplicó.
    let despues = dispositivo.foto();
    let original = antes
        .eventos
        .iter()
        .find(|fila| fila.id == escenario.laptop[0].id)
        .unwrap();
    assert!(despues.eventos.contains(original));
    assert_eq!(despues.eventos.len(), 44);
    assert_eq!(dispositivo.saldo(), d(SALDO_CONVERGIDO));
}

#[test]
fn el_motivo_del_rechazo_nombra_el_campo_que_cambio_y_la_base_no_cambia() {
    let escenario = Escenario26::nuevo();
    let casos: [(&str, Modificacion); 4] = [
        ("tipo", |evento| {
            evento.tipo = "MovimientoCorregido".to_string();
        }),
        ("marca temporal", |evento| {
            evento.registrado_en = "2026-11-10T09:00:01.000000000+00:00".to_string();
        }),
        ("dispositivo", |evento| {
            evento.dispositivo = DISPOSITIVO_TELEFONO.to_string();
        }),
        ("contenido", |evento| {
            evento.datos["monto"] = json!("999.00");
        }),
    ];

    for (campo, modificar) in casos {
        let mut local = escenario.base.clone();
        local.extend(escenario.laptop.clone());
        let dispositivo = Dispositivo::con_eventos(&local);
        let antes = dispositivo.foto();

        let mut alterado = escenario.laptop[0].clone();
        modificar(&mut alterado);
        let resumen = dispositivo.fusionar(&[alterado]);

        assert_eq!(resumen.insertados, 0, "campo {campo}");
        assert_eq!(resumen.rechazados.len(), 1, "campo {campo}");
        assert!(
            resumen.rechazados[0].motivo.contains(campo),
            "el motivo debería mencionar '{campo}': {}",
            resumen.rechazados[0].motivo
        );
        assert_eq!(dispositivo.foto(), antes, "campo {campo}: la base cambió");
    }
}

#[test]
fn un_evento_invalido_se_rechaza_sin_abortar_el_resto_del_lote() {
    let escenario = Escenario26::nuevo();
    let dispositivo = Dispositivo::con_eventos(&escenario.base);

    let sin_id = EventoLibro {
        id: String::new(),
        tipo: "MovimientoRegistrado".to_string(),
        registrado_en: "2026-11-10T09:00:00.000000000+00:00".to_string(),
        dispositivo: DISPOSITIVO_LAPTOP.to_string(),
        datos_canonicos: "{}".to_string(),
    };
    let json_roto = EventoLibro {
        id: "evento-roto".to_string(),
        tipo: "MovimientoRegistrado".to_string(),
        registrado_en: "2026-11-10T09:00:00.000000000+00:00".to_string(),
        dispositivo: DISPOSITIVO_LAPTOP.to_string(),
        datos_canonicos: "{no es json".to_string(),
    };
    let valido = a_evento_libro(&escenario.laptop[0]);

    let resumen = dispositivo.fusionar_libro(&[sin_id, json_roto, valido]);

    assert_eq!(resumen.total_recibidos, 3);
    assert_eq!(resumen.insertados, 1);
    assert_eq!(resumen.rechazados.len(), 2);
    assert_eq!(dispositivo.contar_eventos(), 41);
    assert_eq!(dispositivo.saldo(), d("131.67")); // 151,67 - 20,00
}

// ---------------------------------------------------------------------------------------
// Deduplicación de gastos de grupo (E7): la misma parte bajada en dos dispositivos
// ---------------------------------------------------------------------------------------

const GRUPO: &str = "grupo-viaje";

struct CuentasPrueba;

fn cuenta_dto() -> contrato::cuenta::CuentaDTO {
    contrato::cuenta::CuentaDTO {
        id: CUENTA_EFECTIVO.to_string(),
        nombre: "Efectivo".to_string(),
        tipo: contrato::cuenta::TipoCuentaDTO::Efectivo,
        moneda_codigo: "USD".to_string(),
        saldo: "100.00".to_string(),
        creada_en: "2026-11-01T00:00:00Z".to_string(),
        activa: true,
    }
}

impl RepositorioCuentas for CuentasPrueba {
    fn guardar_cuenta(
        &self,
        _request: &contrato::crear_cuenta::CrearCuentaRequest,
    ) -> Result<contrato::cuenta::CuentaDTO, String> {
        Err("No implementado".to_string())
    }

    fn obtener_cuentas(&self) -> Result<Vec<contrato::cuenta::CuentaDTO>, String> {
        Ok(vec![cuenta_dto()])
    }

    fn obtener_cuenta_por_id(&self, id: &str) -> Result<contrato::cuenta::CuentaDTO, String> {
        self.obtener_cuentas()?
            .into_iter()
            .find(|cuenta| cuenta.id == id)
            .ok_or_else(|| "Cuenta no encontrada".to_string())
    }
}

/// Parte de un gasto de grupo tal como la entrega el servidor.
fn parte_de_grupo(id_evento: &str, monto: &str) -> EventoCambioDTO {
    EventoCambioDTO {
        id_evento: id_evento.to_string(),
        tipo: "GastoDeGrupoRegistrado".to_string(),
        grupo_id: GRUPO.to_string(),
        gasto_id: format!("gasto-{id_evento}"),
        concepto: "Hotel".to_string(),
        fecha: "2026-11-16T00:00:00Z".to_string(),
        moneda: "USD".to_string(),
        monto_parte: monto.to_string(),
        incluye_redondeo: false,
        posicion_cursor: 1,
    }
}

/// El dispositivo "baja" la parte del servidor y la aplica a su cuenta, igual que al
/// sincronizar. Devuelve cuántas partes se aplicaron (0 si ya la tenía).
fn bajar_gasto(dispositivo: &Dispositivo, parte: &EventoCambioDTO) -> usize {
    let repo = RepositorioSincronizacionSqlite::new(bd::abrir(dispositivo.ruta()).unwrap());
    repo.guardar_cuenta_grupo(&CuentaGrupoSeleccionDTO {
        grupo_id: GRUPO.to_string(),
        cuenta_id: CUENTA_EFECTIVO.to_string(),
        moneda: "USD".to_string(),
    })
    .unwrap();

    let movimientos = match preparar_evento(parte, Some("Viaje"), &repo, &CuentasPrueba).unwrap() {
        ResultadoPreparacion::Aplicar {
            movimiento,
            evento,
            efecto_saldo,
        } => vec![AplicacionGastoPlan {
            movimiento,
            evento,
            efecto_saldo,
        }],
        ResultadoPreparacion::YaAplicado => vec![],
        ResultadoPreparacion::Pendiente(_) => {
            panic!("Se esperaba una parte aplicable, no pendiente")
        }
    };
    let plan = LoteSincronizacionPlan {
        nuevo_cursor: 1,
        movimientos,
        pendientes: vec![],
    };
    repo.aplicar_lote_transaccional(&plan).unwrap().len()
}

/// Parte de grupo "normal": sus ids derivados ordenan primero el movimiento y después la
/// marca de gasto aplicado, que es el orden en que la reconstrucción lo necesita.
fn parte_normal() -> EventoCambioDTO {
    parte_de_grupo("evento-cena", "15.00")
}

/// Parte cuyos ids derivados ordenan la marca de aplicado ANTES que su movimiento
/// (el UUID v5 de `GastoDeGrupoAplicado:evento-hotel` es menor que el de
/// `MovimientoRegistrado:evento-hotel`). Con ids de servidor al azar pasa en la mitad
/// de los gastos, porque los dos eventos tienen la misma hora y el mismo dispositivo.
fn parte_con_aplicado_primero() -> EventoCambioDTO {
    parte_de_grupo("evento-hotel", "33.33")
}

#[test]
fn la_misma_parte_de_grupo_bajada_en_dos_dispositivos_queda_una_sola_vez() {
    let escenario = Escenario26::nuevo();
    let laptop = Dispositivo::con_eventos(&escenario.base);
    let telefono = Dispositivo::con_eventos(&escenario.base);
    let parte = parte_normal();

    assert_eq!(bajar_gasto(&laptop, &parte), 1);
    assert_eq!(bajar_gasto(&telefono, &parte), 1);
    assert_eq!(laptop.saldo(), d("136.67")); // 151,67 - 15,00
    assert_eq!(telefono.saldo(), d("136.67"));
    // 40 de la base + el movimiento + la marca de gasto de grupo aplicado.
    assert_eq!(laptop.contar_eventos(), 42);
    assert_eq!(telefono.contar_eventos(), 42);

    // Al unir los libros no aparece nada nuevo ni nada rechazado: es el mismo evento.
    let resumen = telefono.fusionar_libro(&laptop.eventos());
    assert!(
        resumen.rechazados.is_empty(),
        "los eventos derivados deben ser idénticos en los dos dispositivos: {:?}",
        resumen.rechazados
    );
    assert_eq!(resumen.insertados, 0);
    assert_eq!(resumen.ignorados_duplicados, 42);
    assert_eq!(telefono.contar_eventos(), 42);
    assert_eq!(telefono.saldo(), d("136.67"));
    assert_eq!(laptop.foto().eventos, telefono.foto().eventos);
}

/// El teléfono recibe por el libro un gasto de grupo cuyos dos eventos tienen la misma hora y
/// el mismo dispositivo; la marca de aplicado ordena antes que su movimiento. La fusión debe
/// funcionar igual, sin importar cómo ordenen los ids derivados.
#[test]
fn el_orden_de_los_ids_de_un_gasto_de_grupo_no_rompe_la_fusion() {
    let escenario = Escenario26::nuevo();
    let laptop = Dispositivo::con_eventos(&escenario.base);
    let telefono = Dispositivo::con_eventos(&escenario.base);
    assert_eq!(bajar_gasto(&laptop, &parte_con_aplicado_primero()), 1);

    let resultado = telefono.repo.fusionar_lote_transaccional(&laptop.eventos());

    assert!(
        resultado.is_ok(),
        "la fusión falló al recibir un gasto de grupo: {:?}",
        resultado.err()
    );
    assert_eq!(telefono.contar_eventos(), 42);
    assert_eq!(telefono.saldo(), d("118.34")); // 151,67 - 33,33
    assert_eq!(contar_filas(&telefono.lectura, "movimiento_grupo"), 1);
    assert_eq!(
        contar_filas(&telefono.lectura, "eventos_grupo_aplicados"),
        1
    );
}

/// Reconstruir todo desde los eventos tampoco duplica el gasto de grupo, aunque el usuario
/// ya haya elegido la cuenta del grupo (`cuenta_por_grupo` apunta a `cuenta`).
#[test]
fn reconstruir_las_proyecciones_con_una_cuenta_de_grupo_elegida_no_duplica_el_gasto() {
    let telefono = Dispositivo::con_eventos(&Escenario26::nuevo().base);
    assert_eq!(bajar_gasto(&telefono, &parte_normal()), 1);

    let resultado = telefono.repo.reconstruir_proyecciones_desde_eventos();

    assert!(
        resultado.is_ok(),
        "la reconstrucción falló con una cuenta de grupo elegida: {:?}",
        resultado.err()
    );
    assert_eq!(telefono.saldo(), d("136.67"));
    assert_eq!(contar_filas(&telefono.lectura, "movimiento_grupo"), 1);
    assert_eq!(
        contar_filas(&telefono.lectura, "eventos_grupo_aplicados"),
        1
    );
}

/// Caso real: la laptop ya bajó un gasto de grupo (eligió su cuenta) y luego recibe del
/// teléfono 2 eventos nuevos. La fusión debe funcionar y conservar el gasto una sola vez.
#[test]
fn fusionar_eventos_nuevos_en_un_dispositivo_con_cuenta_de_grupo_elegida() {
    let escenario = Escenario26::nuevo();
    let laptop = Dispositivo::con_eventos(&escenario.base);
    assert_eq!(bajar_gasto(&laptop, &parte_normal()), 1);

    let lote: Vec<EventoLibro> = escenario.telefono.iter().map(a_evento_libro).collect();
    let resultado = laptop.repo.fusionar_lote_transaccional(&lote);

    assert!(
        resultado.is_ok(),
        "la fusión falló en un dispositivo con cuenta de grupo elegida: {:?}",
        resultado.err()
    );
    assert_eq!(laptop.contar_eventos(), 44); // 40 + 2 del gasto de grupo + 2 del teléfono
    assert_eq!(laptop.saldo(), d("176.67")); // 151,67 - 15,00 + 50,00 - 10,00
    assert_eq!(contar_filas(&laptop.lectura, "movimiento_grupo"), 1);
}

#[test]
fn si_el_libro_ya_trae_la_parte_el_dispositivo_no_la_aplica_otra_vez_al_bajarla() {
    let escenario = Escenario26::nuevo();
    let laptop = Dispositivo::con_eventos(&escenario.base);
    let telefono = Dispositivo::con_eventos(&escenario.base);
    let parte = parte_normal();

    // La laptop baja la parte; el teléfono la recibe primero por el libro...
    assert_eq!(bajar_gasto(&laptop, &parte), 1);
    let resumen = telefono.fusionar_libro(&laptop.eventos());
    assert_eq!(resumen.insertados, 2);
    assert_eq!(telefono.saldo(), d("136.67"));

    // ...y después la baja también del servidor: no debe descontarse otra vez.
    assert_eq!(bajar_gasto(&telefono, &parte), 0);
    assert_eq!(telefono.saldo(), d("136.67"));
    assert_eq!(telefono.contar_eventos(), 42);
    assert_eq!(contar_filas(&telefono.lectura, "movimiento_grupo"), 1);
}

// ---------------------------------------------------------------------------------------
// Correcciones: un MovimientoCorregido hecho en un dispositivo debe poder fusionarse
// ---------------------------------------------------------------------------------------

/// El evento tiene los campos que escribe `guardar_ajuste` en la app: `movimiento_original_id`,
/// `cuenta_id`, `tipo_original` y `variacion_monto`. Un gasto de 20,00 corregido a 15,00 deja
/// una variación de -5,00 y el saldo sube 5,00.
#[test]
fn una_correccion_hecha_en_un_dispositivo_se_fusiona_en_el_otro() {
    let escenario = Escenario26::nuevo();
    let movimiento_original = escenario.laptop[0].datos["id"]
        .as_str()
        .unwrap()
        .to_string();
    let correccion = EventoPrueba {
        id: "00000000-0000-4000-8000-000000002601".to_string(),
        tipo: "MovimientoCorregido".to_string(),
        registrado_en: "2026-11-11T09:00:00.000000000+00:00".to_string(),
        dispositivo: DISPOSITIVO_LAPTOP.to_string(),
        datos: json!({
            "id": "00000000-0000-4000-8000-000000002602",
            "movimiento_original_id": movimiento_original,
            "cuenta_id": CUENTA_EFECTIVO,
            "tipo_original": "Gasto",
            "variacion_monto": "-5.00",
        }),
    };

    let telefono = Dispositivo::con_eventos(&escenario.base);
    let lote: Vec<EventoLibro> = [&escenario.laptop[0], &correccion]
        .into_iter()
        .map(a_evento_libro)
        .collect();
    let resultado = telefono.repo.fusionar_lote_transaccional(&lote);

    assert!(
        resultado.is_ok(),
        "la fusión falló con una corrección real: {:?}",
        resultado.err()
    );
    assert_eq!(telefono.contar_eventos(), 42);
    // 151,67 - 20,00 + 5,00
    assert_eq!(telefono.saldo(), d("136.67"));
    assert_eq!(telefono.saldo_recalculado(), d("136.67"));
}

// ---------------------------------------------------------------------------------------
// Escenario 2.6 completo por carpeta compartida: convergencia e informes de sincronización
// ---------------------------------------------------------------------------------------

const FRASE: &str = "frase-de-prueba-123";

/// Cifrado de mentira (XOR con la frase) para no pagar Argon2id en cada lote: aquí se
/// prueba la fusión y el transporte, no la criptografía.
struct CifradorDePrueba;

impl CifradorRespaldo for CifradorDePrueba {
    fn cifrar(&self, datos_planos: &[u8], frase_paso: &str) -> Result<Vec<u8>, String> {
        if frase_paso.is_empty() {
            return Err("Frase vacía".to_string());
        }
        Ok(datos_planos
            .iter()
            .zip(frase_paso.as_bytes().iter().cycle())
            .map(|(dato, clave)| dato ^ clave)
            .collect())
    }

    fn descifrar(&self, paquete_cifrado: &[u8], frase_paso: &str) -> Result<Vec<u8>, String> {
        self.cifrar(paquete_cifrado, frase_paso)
    }
}

struct RelojFijo;

impl Reloj for RelojFijo {
    fn ahora(&self) -> String {
        "2026-11-26T12:00:00Z".to_string()
    }
}

/// Sincroniza y devuelve (enviados, recibidos).
fn sincronizar(carpeta: &Path, dispositivo: &Dispositivo) -> (usize, usize) {
    let reporte = sincronizar_carpeta_libro::ejecutar(
        carpeta,
        FRASE,
        &dispositivo.repo,
        &SerializadorLibroPlano::new(),
        &CifradorDePrueba,
        &RelojFijo,
    )
    .unwrap();
    assert!(
        reporte.rechazados.is_empty(),
        "no debería haber rechazos: {:?}",
        reporte.rechazados
    );
    (reporte.eventos_enviados, reporte.eventos_recibidos)
}

/// Ambos dispositivos parten con los 40 eventos ya sincronizados; después, sin conexión,
/// cada uno registra sus 2 movimientos.
fn escenario_26_por_carpeta(carpeta: &Path) -> (Dispositivo, Dispositivo) {
    let escenario = Escenario26::nuevo();
    let laptop = Dispositivo::con_eventos(&escenario.base);
    let telefono = Dispositivo::con_eventos(&escenario.base);
    sincronizar(carpeta, &laptop);
    sincronizar(carpeta, &telefono);

    laptop.fusionar(&escenario.laptop);
    telefono.fusionar(&escenario.telefono);
    assert_eq!(laptop.saldo(), d(SALDO_SOLO_LAPTOP));
    assert_eq!(telefono.saldo(), d(SALDO_SOLO_TELEFONO));
    (laptop, telefono)
}

#[test]
fn por_carpeta_compartida_laptop_primero_o_telefono_primero_converge_a_44_y_166_67() {
    let mut fotos = Vec::new();
    for laptop_primero in [true, false] {
        let carpeta = CarpetaTemporal::nueva("convergencia");
        let (laptop, telefono) = escenario_26_por_carpeta(&carpeta.0);

        for _ in 0..2 {
            if laptop_primero {
                sincronizar(&carpeta.0, &laptop);
                sincronizar(&carpeta.0, &telefono);
            } else {
                sincronizar(&carpeta.0, &telefono);
                sincronizar(&carpeta.0, &laptop);
            }
        }

        for dispositivo in [&laptop, &telefono] {
            assert_eq!(dispositivo.contar_eventos(), EVENTOS_CONVERGIDOS);
            assert_eq!(dispositivo.saldo(), d(SALDO_CONVERGIDO));
            assert_eq!(dispositivo.saldo_recalculado(), d(SALDO_CONVERGIDO));
        }
        assert_eq!(laptop.foto(), telefono.foto());
        fotos.push(laptop.foto());
    }
    assert_eq!(
        fotos[0], fotos[1],
        "los dos órdenes deben dar el mismo libro"
    );
}

/// Informes de la especificación (§2.6): la laptop envía 2 y recibe 0; el teléfono envía 2 y
/// recibe 2; la laptop vuelve a sincronizar y recibe 2 sin enviar nada; luego todo en 0.
#[test]
fn los_informes_de_sincronizacion_del_escenario_26_coinciden_con_la_especificacion() {
    let carpeta = CarpetaTemporal::nueva("informes");
    let (laptop, telefono) = escenario_26_por_carpeta(&carpeta.0);

    let obtenidos = vec![
        sincronizar(&carpeta.0, &laptop),
        sincronizar(&carpeta.0, &telefono),
        sincronizar(&carpeta.0, &laptop),
        sincronizar(&carpeta.0, &telefono),
        sincronizar(&carpeta.0, &laptop),
    ];
    let esperados = vec![(2, 0), (2, 2), (0, 2), (0, 0), (0, 0)];

    assert_eq!(
        obtenidos, esperados,
        "(enviados, recibidos) en cada sincronización: laptop, teléfono, laptop, teléfono, laptop"
    );
}
