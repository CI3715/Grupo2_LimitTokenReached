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
        Self {
            fecha_recibida: Mutex::new(None),
            saldos: vec![SaldoCuentaDTO {
                cuenta_id: "cuenta-1".to_string(),
                nombre_cuenta: "Cuenta USD".to_string(),
                moneda: "USD".to_string(),
                saldo: "100.00".to_string(),
            }],
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
}

impl RepositorioTasasPrueba {
    fn nuevo() -> Self {
        Self {
            fecha_recibida: Mutex::new(None),
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
        Ok(Vec::new())
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
        Ok(vec![MonedaDTO {
            codigo: "USD".to_string(),
            nombre: "Dólar Estadounidense".to_string(),
            simbolo: "$".to_string(),
            decimales: 2,
            es_del_catalogo: true,
        }])
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
