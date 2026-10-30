use aplicacion::{
    casos_uso::registrar_tasa::ejecutar,
    puertos::{Reloj, RepositorioMonedas, RepositorioTasas},
};
use chrono::NaiveDate;
use contrato::{
    error_validacion::ErrorValidacionDTO,
    moneda::{CrearMonedaRequest, MonedaDTO},
    tasa::{RegistrarTasaRequest, TasaDTO},
};
use std::sync::Mutex;

struct RepositorioMonedasPrueba {
    monedas: Vec<MonedaDTO>,
}

impl RepositorioMonedasPrueba {
    fn nuevo() -> Self {
        Self {
            monedas: vec![
                MonedaDTO {
                    codigo: "VES".to_string(),
                    nombre: "Bolívar".to_string(),
                    simbolo: "Bs.".to_string(),
                    decimales: 2,
                    es_del_catalogo: true,
                },
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
                MonedaDTO {
                    codigo: "USDT".to_string(),
                    nombre: "Tether".to_string(),
                    simbolo: "₮".to_string(),
                    decimales: 6,
                    es_del_catalogo: true,
                },
            ],
        }
    }
}

impl RepositorioMonedas for RepositorioMonedasPrueba {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String> {
        Ok(self.monedas.clone())
    }

    fn guardar_moneda(&self, request: &CrearMonedaRequest) -> Result<MonedaDTO, String> {
        let moneda = MonedaDTO {
            codigo: request.codigo.clone(),
            nombre: request.nombre.clone(),
            simbolo: request.simbolo.clone(),
            decimales: request.decimales,
            es_del_catalogo: false,
        };

        Ok(moneda)
    }

    fn eliminar_moneda(&self, _codigo: &str) -> Result<(), String> {
        Err("No se usa eliminar_moneda en esta prueba.".to_string())
    }
}

struct RepositorioTasasPrueba {
    guardadas: Mutex<Vec<TasaDTO>>,
}

impl RepositorioTasasPrueba {
    fn nuevo() -> Self {
        Self {
            guardadas: Mutex::new(Vec::new()),
        }
    }
}

impl RepositorioTasas for RepositorioTasasPrueba {
    fn guardar_tasa(&self, request: &RegistrarTasaRequest) -> Result<TasaDTO, String> {
        let tasa = TasaDTO {
            id: "tasa-prueba-1".to_string(),
            moneda_origen: request.moneda_origen.clone(),
            moneda_destino: request.moneda_destino.clone(),
            valor: request.valor.clone(),
            fecha: request.fecha.clone(),
            fuente: request.fuente.clone(),
            eliminada: false,
        };

        self.guardadas
            .lock()
            .map_err(|error| error.to_string())?
            .push(tasa.clone());

        Ok(tasa)
    }

    fn obtener_tasas_hasta(&self, _fecha: NaiveDate) -> Result<Vec<TasaDTO>, String> {
        Ok(Vec::new())
    }

    fn obtener_historial_tasas_hasta(&self, _fecha: NaiveDate) -> Result<Vec<TasaDTO>, String> {
        Ok(Vec::new())
    }

    fn eliminar_tasa(&self, _tasa_id: &str) -> Result<(), String> {
        Ok(())
    }
}

struct RelojFijo;

impl Reloj for RelojFijo {
    fn ahora(&self) -> String {
        "2026-10-28T12:00:00Z".to_string()
    }
}

fn request_con_valor(valor: &str) -> RegistrarTasaRequest {
    RegistrarTasaRequest {
        moneda_origen: "USD".to_string(),
        moneda_destino: "VES".to_string(),
        valor: valor.to_string(),
        fecha: "2026-10-28".to_string(),
        fuente: "BCV".to_string(),
    }
}

async fn ejecutar_con_valor(valor: &str) -> Result<TasaDTO, ErrorValidacionDTO> {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    ejecutar(request_con_valor(valor), &repo_tasas, &repo_monedas, &reloj).await
}

#[tokio::test]
async fn acepta_una_tasa_con_hasta_ocho_decimales() {
    let resultado = ejecutar_con_valor("887.12345678").await;

    assert!(resultado.is_ok());
    assert_eq!(resultado.unwrap().valor, "887.12345678");
}

#[tokio::test]
async fn acepta_una_tasa_con_ceros_finales() {
    let resultado = ejecutar_con_valor("887.123456780000").await;

    assert!(resultado.is_ok());
}

#[tokio::test]
async fn rechaza_una_tasa_con_mas_de_ocho_decimales() {
    let resultado = ejecutar_con_valor("887.123456789").await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "valor");
    assert_eq!(error.mensaje, "La tasa no puede tener más de 8 decimales.");
}

#[tokio::test]
async fn rechaza_el_valor_cero() {
    let resultado = ejecutar_con_valor("0").await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "valor");
    assert_eq!(
        error.mensaje,
        "El valor de la tasa debe ser mayor que cero."
    );
}

#[tokio::test]
async fn rechaza_el_valor_negativo() {
    let resultado = ejecutar_con_valor("-1.25").await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "valor");
    assert_eq!(
        error.mensaje,
        "El valor de la tasa debe ser mayor que cero."
    );
}

#[tokio::test]
async fn rechaza_un_valor_decimal_invalido() {
    let resultado = ejecutar_con_valor("no-es-un-numero").await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "valor");
    assert_eq!(error.mensaje, "El valor debe ser un número decimal válido.");
}

#[tokio::test]
async fn rechaza_monedas_iguales() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    let mut request = request_con_valor("100");
    request.moneda_destino = "USD".to_string();

    let resultado = ejecutar(request, &repo_tasas, &repo_monedas, &reloj).await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "moneda_destino");
    assert_eq!(error.mensaje, "Las monedas deben ser diferentes.");
}

#[tokio::test]
async fn rechaza_una_moneda_que_no_existe_en_el_catalogo() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    let mut request = request_con_valor("100");
    request.moneda_destino = "ABC".to_string();

    let resultado = ejecutar(request, &repo_tasas, &repo_monedas, &reloj).await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "moneda");
    assert_eq!(error.mensaje, "La moneda 'ABC' no existe en el catálogo.");
}

#[tokio::test]
async fn normaliza_los_codigos_de_moneda() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    let mut request = request_con_valor("100");
    request.moneda_origen = " usd ".to_string();
    request.moneda_destino = " ves ".to_string();

    let resultado = ejecutar(request, &repo_tasas, &repo_monedas, &reloj)
        .await
        .unwrap();

    assert_eq!(resultado.moneda_origen, "USD");
    assert_eq!(resultado.moneda_destino, "VES");
}

#[tokio::test]
async fn rechaza_una_fuente_vacia() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    let mut request = request_con_valor("100");
    request.fuente = "   ".to_string();

    let resultado = ejecutar(request, &repo_tasas, &repo_monedas, &reloj).await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "fuente");
    assert_eq!(error.mensaje, "La fuente no puede estar vacía.");
}

#[tokio::test]
async fn rechaza_una_fecha_futura() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    let mut request = request_con_valor("100");
    request.fecha = "2026-10-29".to_string();

    let resultado = ejecutar(request, &repo_tasas, &repo_monedas, &reloj).await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "fecha");
    assert_eq!(error.mensaje, "La fecha de la tasa no puede ser futura.");
}

#[tokio::test]
async fn rechaza_una_fecha_con_formato_invalido() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    let mut request = request_con_valor("100");
    request.fecha = "28-10-2026".to_string();

    let resultado = ejecutar(request, &repo_tasas, &repo_monedas, &reloj).await;

    let error = resultado.unwrap_err();

    assert_eq!(error.campo, "fecha");
    assert_eq!(error.mensaje, "Formato inválido. Use AAAA-MM-DD.");
}

#[tokio::test]
async fn rechaza_una_fecha_vacia_indicando_el_campo() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;
    let mut request = request_con_valor("100");
    request.fecha.clear();

    let error = ejecutar(request, &repo_tasas, &repo_monedas, &reloj)
        .await
        .expect_err("La fecha vacía debe rechazarse");

    assert_eq!(error.campo, "fecha");
    assert_eq!(error.mensaje, "La fecha es requerida.");
}

#[tokio::test]
async fn guarda_la_tasa_normalizada() {
    let repo_tasas = RepositorioTasasPrueba::nuevo();
    let repo_monedas = RepositorioMonedasPrueba::nuevo();
    let reloj = RelojFijo;

    let mut request = request_con_valor(" 887.25 ");
    request.fuente = " BCV ".to_string();

    let resultado = ejecutar(request, &repo_tasas, &repo_monedas, &reloj)
        .await
        .unwrap();

    assert_eq!(resultado.valor, "887.25");
    assert_eq!(resultado.fuente, "BCV");
}
