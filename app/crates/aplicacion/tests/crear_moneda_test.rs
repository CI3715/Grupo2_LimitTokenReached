use aplicacion::{
    casos_uso::crear_moneda,
    puertos::RepositorioMonedas,
};
use contrato::{
    error_validacion::ErrorValidacionDTO,
    moneda::{CrearMonedaRequest, MonedaDTO},
};
use std::sync::Mutex;

#[derive(Default)]
struct RepositorioMonedasPrueba {
    monedas: Mutex<Vec<MonedaDTO>>,
}

impl RepositorioMonedasPrueba {
    fn nuevo() -> Self {
        Self {
            monedas: Mutex::new(vec![
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
            ]),
        }
    }
}

impl RepositorioMonedas for RepositorioMonedasPrueba {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String> {
        self.monedas
            .lock()
            .map(|monedas| monedas.clone())
            .map_err(|error| error.to_string())
    }

    fn guardar_moneda(
        &self,
        request: &CrearMonedaRequest,
    ) -> Result<MonedaDTO, String> {
        let moneda = MonedaDTO {
            codigo: request.codigo.clone(),
            nombre: request.nombre.clone(),
            simbolo: request.simbolo.clone(),
            decimales: request.decimales,
            es_del_catalogo: false,
        };

        self.monedas
            .lock()
            .map_err(|error| error.to_string())?
            .push(moneda.clone());

        Ok(moneda)
    }
}

fn request_valida() -> CrearMonedaRequest {
    CrearMonedaRequest {
        codigo: "GBP".to_string(),
        nombre: "Libra Esterlina".to_string(),
        simbolo: "£".to_string(),
        decimales: 2,
    }
}

#[tokio::test]
async fn crea_moneda_valida() {
    let repo = RepositorioMonedasPrueba::nuevo();

    let resultado = crear_moneda::ejecutar(request_valida(), &repo).await;

    assert!(resultado.is_ok());

    let moneda = resultado.unwrap();
    assert_eq!(moneda.codigo, "GBP");
    assert_eq!(moneda.nombre, "Libra Esterlina");
    assert_eq!(moneda.simbolo, "£");
    assert_eq!(moneda.decimales, 2);
    assert!(!moneda.es_del_catalogo);
}

#[tokio::test]
async fn rechaza_codigo_corto() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.codigo = "US".to_string();

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "codigo");
}

#[tokio::test]
async fn rechaza_codigo_largo() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.codigo = "DOLLAR".to_string();

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "codigo");
}

#[tokio::test]
async fn rechaza_codigo_en_minusculas() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.codigo = "gbp".to_string();

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "codigo");
}

#[tokio::test]
async fn rechaza_codigo_duplicado() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.codigo = "USD".to_string();

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "codigo");
}

#[tokio::test]
async fn rechaza_moneda_de_fabrica() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.codigo = "EUR".to_string();

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "codigo");
}

#[tokio::test]
async fn rechaza_mas_de_ocho_decimales() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.decimales = 9;

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "decimales");
}

#[tokio::test]
async fn rechaza_nombre_vacio() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.nombre = "   ".to_string();

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "nombre");
}

#[tokio::test]
async fn rechaza_simbolo_vacio() {
    let repo = RepositorioMonedasPrueba::nuevo();
    let mut request = request_valida();
    request.simbolo = "   ".to_string();

    let resultado = crear_moneda::ejecutar(request, &repo).await;

    assert_eq!(resultado.unwrap_err().campo, "simbolo");
}