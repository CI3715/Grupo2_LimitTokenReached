use aplicacion::{casos_uso::eliminar_moneda, puertos::RepositorioMonedas};
use contrato::moneda::{CrearMonedaRequest, EliminarMonedaRequest, MonedaDTO};
use std::sync::Mutex;

struct RepositorioMonedasPrueba {
    monedas: Mutex<Vec<MonedaDTO>>,
}

impl RepositorioMonedasPrueba {
    fn nuevo() -> Self {
        Self {
            monedas: Mutex::new(vec![
                MonedaDTO {
                    codigo: "USD".to_string(),
                    nombre: "Dólar".to_string(),
                    simbolo: "$".to_string(),
                    decimales: 2,
                    es_del_catalogo: true,
                },
                MonedaDTO {
                    codigo: "GBP".to_string(),
                    nombre: "Libra Esterlina".to_string(),
                    simbolo: "£".to_string(),
                    decimales: 2,
                    es_del_catalogo: false,
                },
            ]),
        }
    }
}

impl RepositorioMonedas for RepositorioMonedasPrueba {
    fn obtener_monedas(&self) -> Result<Vec<MonedaDTO>, String> {
        Ok(self.monedas.lock().unwrap().clone())
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

    fn eliminar_moneda(&self, codigo: &str) -> Result<(), String> {
        let mut monedas = self.monedas.lock().unwrap();

        let posicion = monedas
            .iter()
            .position(|moneda| moneda.codigo == codigo)
            .ok_or_else(|| format!("No existe la moneda '{codigo}'."))?;

        monedas.remove(posicion);
        Ok(())
    }
}

#[tokio::test]
async fn rechaza_eliminar_moneda_usd() {
    let repo = RepositorioMonedasPrueba::nuevo();

    let resultado = eliminar_moneda::ejecutar(
        EliminarMonedaRequest {
            codigo: "USD".to_string(),
        },
        &repo,
    )
    .await;

    let error = resultado.expect_err("USD no debe poder eliminarse");

    assert_eq!(error.campo, "codigo");
    assert_eq!(
        error.mensaje,
        "Las monedas de fábrica no se pueden eliminar."
    );
}

#[tokio::test]
async fn rechaza_eliminar_las_cuatro_monedas_de_fabrica() {
    for codigo in ["VES", "USD", "EUR", "USDT"] {
        let repo = RepositorioMonedasPrueba::nuevo();

        let resultado = eliminar_moneda::ejecutar(
            EliminarMonedaRequest {
                codigo: codigo.to_string(),
            },
            &repo,
        )
        .await;

        let error = resultado.expect_err("La moneda de fábrica no debe eliminarse");

        assert_eq!(error.campo, "codigo");
        assert_eq!(
            error.mensaje,
            "Las monedas de fábrica no se pueden eliminar."
        );
    }
}

#[tokio::test]
async fn elimina_moneda_propia_no_utilizada() {
    let repo = RepositorioMonedasPrueba::nuevo();

    let resultado = eliminar_moneda::ejecutar(
        EliminarMonedaRequest {
            codigo: "GBP".to_string(),
        },
        &repo,
    )
    .await;

    assert!(resultado.is_ok());

    let monedas = repo.obtener_monedas().unwrap();
    assert!(!monedas.iter().any(|moneda| moneda.codigo == "GBP"));
}

#[tokio::test]
async fn rechaza_codigo_vacio() {
    let repo = RepositorioMonedasPrueba::nuevo();

    let resultado = eliminar_moneda::ejecutar(
        EliminarMonedaRequest {
            codigo: "   ".to_string(),
        },
        &repo,
    )
    .await;

    let error = resultado.expect_err("El código vacío debe rechazarse");

    assert_eq!(error.campo, "codigo");
}
