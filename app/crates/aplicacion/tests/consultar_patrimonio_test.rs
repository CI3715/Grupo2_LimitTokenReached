use aplicacion::{
    casos_uso::consultar_patrimonio,
    puertos::{
        Reloj, RepositorioMonedas, RepositorioPatrimonio, RepositorioPreferencias, RepositorioTasas,
    },
};
use chrono::NaiveDate;
use contrato::{
    moneda::{CrearMonedaRequest, MonedaDTO},
    patrimonio::{ConsultarPatrimonioRequest, SaldoCuentaDTO},
    tasa::{RegistrarTasaRequest, TasaDTO},
};
use std::sync::Mutex;

struct RelojFijo;

impl Reloj for RelojFijo {
    fn ahora(&self) -> String {
        "2026-10-28T12:00:00Z".to_string()
    }
}

struct RepositorioPatrimonioPrueba {
    fecha_recibida: Mutex<Option<NaiveDate>>,
    saldos: Vec<SaldoCuentaDTO>,
}

impl RepositorioPatrimonioPrueba {
    fn nuevo() -> Self {
        Self::con_saldos(vec![SaldoCuentaDTO {
            cuenta_id: "cuenta-1".to_string(),
            nombre_cuenta: "Cuenta EUR".to_string(),
            moneda: "EUR".to_string(),
            saldo: "100.00".to_string(),
        }])
    }

    fn con_saldos(saldos: Vec<SaldoCuentaDTO>) -> Self {
        Self {
            fecha_recibida: Mutex::new(None),
            saldos,
        }
    }

    fn fecha_recibida(&self) -> NaiveDate {
        self.fecha_recibida
            .lock()
            .unwrap()
            .expect("El repositorio debe recibir una fecha")
    }
}

impl RepositorioPatrimonio for RepositorioPatrimonioPrueba {
    fn obtener_saldos_a_fecha(&self, fecha: NaiveDate) -> Result<Vec<SaldoCuentaDTO>, String> {
        *self.fecha_recibida.lock().unwrap() = Some(fecha);
        Ok(self.saldos.clone())
    }
}

struct RepositorioTasasPrueba {
    fecha_recibida: Mutex<Option<NaiveDate>>,
    tasas: Vec<TasaDTO>,
}

impl RepositorioTasasPrueba {
    fn nuevo() -> Self {
        Self::con_tasas(Vec::new())
    }

    fn con_tasas(tasas: Vec<TasaDTO>) -> Self {
        Self {
            fecha_recibida: Mutex::new(None),
            tasas,
        }
    }

    fn fecha_recibida(&self) -> NaiveDate {
        self.fecha_recibida
            .lock()
            .unwrap()
            .expect("El repositorio debe recibir una fecha")
    }
}

impl RepositorioTasas for RepositorioTasasPrueba {
    fn guardar_tasa(&self, _request: &RegistrarTasaRequest) -> Result<TasaDTO, String> {
        Err("No se usa guardar_tasa en esta prueba.".to_string())
    }

    fn obtener_tasas_hasta(&self, fecha: NaiveDate) -> Result<Vec<TasaDTO>, String> {
        *self.fecha_recibida.lock().unwrap() = Some(fecha);
        let mut tasas: Vec<_> = self
            .tasas
            .iter()
            .filter(|tasa| {
                !tasa.eliminada
                    && NaiveDate::parse_from_str(&tasa.fecha, "%Y-%m-%d")
                        .is_ok_and(|fecha_tasa| fecha_tasa <= fecha)
            })
            .cloned()
            .collect();
        tasas.sort_by(|left, right| {
            right.fecha.cmp(&left.fecha).then_with(|| {
                right
                    .id
                    .parse::<i64>()
                    .unwrap_or_default()
                    .cmp(&left.id.parse::<i64>().unwrap_or_default())
            })
        });
        Ok(tasas)
    }

    fn obtener_historial_tasas_hasta(&self, _fecha: NaiveDate) -> Result<Vec<TasaDTO>, String> {
        Ok(Vec::new())
    }

    fn eliminar_tasa(&self, _tasa_id: &str) -> Result<(), String> {
        Err("No se usa eliminar_tasa en esta prueba.".to_string())
    }
}

struct RepositorioMonedasPrueba;

impl RepositorioMonedas for RepositorioMonedasPrueba {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String> {
        Ok(vec![
            MonedaDTO {
                codigo: "USD".to_string(),
                nombre: "Dólar Estadounidense".to_string(),
                simbolo: "$".to_string(),
                decimales: 2,
                es_del_catalogo: true,
            },
            MonedaDTO {
                codigo: "EUR".to_string(),
                nombre: "Euro".to_string(),
                simbolo: "€".to_string(),
                decimales: 2,
                es_del_catalogo: true,
            },
        ])
    }

    fn guardar_moneda(&self, request: &CrearMonedaRequest) -> Result<MonedaDTO, String> {
        Ok(MonedaDTO {
            codigo: request.codigo.clone(),
            nombre: request.nombre.clone(),
            simbolo: request.simbolo.clone(),
            decimales: request.decimales,
            es_del_catalogo: false,
        })
    }

    fn eliminar_moneda(&self, _codigo: &str) -> Result<(), String> {
        Err("No se usa eliminar_moneda en esta prueba.".to_string())
    }
}

struct RepositorioPreferenciasPrueba;

impl RepositorioPreferencias for RepositorioPreferenciasPrueba {
    fn obtener_moneda_base(&self) -> Result<String, String> {
        Ok("USD".to_string())
    }

    fn guardar_moneda_base(&self, _codigo: &str) -> Result<(), String> {
        Ok(())
    }
}

async fn consultar_con_fecha(
    fecha_consulta: &str,
    repo_patrimonio: &RepositorioPatrimonioPrueba,
    repo_tasas: &RepositorioTasasPrueba,
) -> Result<
    contrato::patrimonio::ConsultarPatrimonioResponse,
    contrato::error_validacion::ErrorValidacionDTO,
> {
    consultar_patrimonio::ejecutar(
        ConsultarPatrimonioRequest {
            fecha_consulta: fecha_consulta.to_string(),
        },
        repo_patrimonio,
        repo_tasas,
        &RepositorioPreferenciasPrueba,
        &RepositorioMonedasPrueba,
        &RelojFijo,
    )
    .await
}

fn tasa(id: &str, valor: &str, fecha: &str, origen: &str, destino: &str) -> TasaDTO {
    TasaDTO {
        id: id.to_string(),
        moneda_origen: origen.to_string(),
        moneda_destino: destino.to_string(),
        valor: valor.to_string(),
        fecha: fecha.to_string(),
        fuente: "prueba".to_string(),
        eliminada: false,
    }
}

#[tokio::test]
async fn fecha_vacia_usa_el_dia_actual() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::nuevo();

    let resultado = consultar_con_fecha("", &repo_patrimonio, &repo_tasas).await;

    assert!(resultado.is_ok());
    assert_eq!(
        repo_patrimonio.fecha_recibida(),
        NaiveDate::from_ymd_opt(2026, 10, 28).unwrap()
    );
    assert_eq!(
        repo_tasas.fecha_recibida(),
        NaiveDate::from_ymd_opt(2026, 10, 28).unwrap()
    );
}

#[tokio::test]
async fn espacios_en_la_fecha_tambien_usan_el_dia_actual() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::nuevo();

    let resultado = consultar_con_fecha("   ", &repo_patrimonio, &repo_tasas).await;

    assert!(resultado.is_ok());
    assert_eq!(
        repo_patrimonio.fecha_recibida(),
        NaiveDate::from_ymd_opt(2026, 10, 28).unwrap()
    );
}

#[tokio::test]
async fn fecha_historica_se_conserva() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::nuevo();

    let resultado = consultar_con_fecha("2026-10-27", &repo_patrimonio, &repo_tasas).await;

    assert!(resultado.is_ok());
    assert_eq!(
        repo_patrimonio.fecha_recibida(),
        NaiveDate::from_ymd_opt(2026, 10, 27).unwrap()
    );
    assert_eq!(
        repo_tasas.fecha_recibida(),
        NaiveDate::from_ymd_opt(2026, 10, 27).unwrap()
    );
}

#[tokio::test]
async fn fecha_futura_se_rechaza() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::nuevo();

    let resultado = consultar_con_fecha("2026-10-29", &repo_patrimonio, &repo_tasas).await;

    let error = resultado.expect_err("Una fecha futura debe rechazarse");

    assert_eq!(error.campo, "fecha_consulta");
    assert_eq!(error.mensaje, "La fecha de consulta no puede ser futura.");
}

#[tokio::test]
async fn formato_de_fecha_invalido_se_rechaza() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::nuevo();

    let resultado = consultar_con_fecha("28/10/2026", &repo_patrimonio, &repo_tasas).await;

    let error = resultado.expect_err("El formato inválido debe rechazarse");

    assert_eq!(error.campo, "fecha_consulta");
    assert_eq!(error.mensaje, "Formato inválido. Use AAAA-MM-DD.");
}

#[tokio::test]
async fn informa_tasa_faltante_y_no_calcula_total() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::nuevo();

    let respuesta = consultar_con_fecha("2026-10-28", &repo_patrimonio, &repo_tasas)
        .await
        .unwrap();

    assert_eq!(respuesta.total_consolidado, None);
    assert_eq!(respuesta.subtotales_por_moneda.len(), 1);
    assert_eq!(respuesta.tasas_faltantes.len(), 1);

    let faltante = &respuesta.tasas_faltantes[0];

    assert_eq!(faltante.moneda_origen, "EUR");
    assert_eq!(faltante.moneda_destino, "USD");
    assert_eq!(faltante.fecha_limite, "2026-10-28");
}

#[tokio::test]
async fn usa_la_tasa_mas_reciente_hasta_la_fecha_consultada() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::con_tasas(vec![
        tasa("1", "2", "2026-10-26", "EUR", "USD"),
        tasa("2", "3", "2026-10-27", "EUR", "USD"),
        tasa("3", "4", "2026-10-29", "EUR", "USD"),
    ]);

    let respuesta = consultar_con_fecha("2026-10-28", &repo_patrimonio, &repo_tasas)
        .await
        .unwrap();

    assert_eq!(respuesta.total_consolidado.as_deref(), Some("300"));
    assert_eq!(respuesta.tasas_usadas.len(), 1);
    assert_eq!(respuesta.tasas_usadas[0].valor, "3");
    assert_eq!(respuesta.tasas_usadas[0].fecha, "2026-10-27");
}

#[tokio::test]
async fn usa_la_inversa_cuando_no_hay_tasa_directa() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::nuevo();
    let repo_tasas = RepositorioTasasPrueba::con_tasas(vec![tasa(
        "1",
        "2",
        "2026-10-27",
        "USD",
        "EUR",
    )]);

    let respuesta = consultar_con_fecha("2026-10-28", &repo_patrimonio, &repo_tasas)
        .await
        .unwrap();

    assert_eq!(respuesta.total_consolidado.as_deref(), Some("50"));
    assert!(respuesta.tasas_usadas[0].es_inversa);
}

#[tokio::test]
async fn suma_conversiones_antes_de_redondear_el_total() {
    let repo_patrimonio = RepositorioPatrimonioPrueba::con_saldos(vec![
        SaldoCuentaDTO {
            cuenta_id: "cuenta-1".to_string(),
            nombre_cuenta: "Cuenta EUR 1".to_string(),
            moneda: "EUR".to_string(),
            saldo: "0.01".to_string(),
        },
        SaldoCuentaDTO {
            cuenta_id: "cuenta-2".to_string(),
            nombre_cuenta: "Cuenta EUR 2".to_string(),
            moneda: "EUR".to_string(),
            saldo: "0.01".to_string(),
        },
    ]);
    let repo_tasas = RepositorioTasasPrueba::con_tasas(vec![tasa(
        "1",
        "0.5",
        "2026-10-27",
        "EUR",
        "USD",
    )]);

    let respuesta = consultar_con_fecha("2026-10-28", &repo_patrimonio, &repo_tasas)
        .await
        .unwrap();

    assert_eq!(respuesta.total_consolidado.as_deref(), Some("0.01"));
}
