use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::categoria::{Categoria, TipoCategoria};
use crate::errores::ErrorDominio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoMovimiento {
    Ingreso,
    Gasto,
}

impl TipoMovimiento {
    fn categoria_esperada(self) -> TipoCategoria {
        match self {
            Self::Ingreso => TipoCategoria::Ingreso,
            Self::Gasto => TipoCategoria::Gasto,
        }
    }
}

/// Reglas de la moneda de la cuenta (la moneda nunca la elige el usuario).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonedaCuenta {
    pub codigo: String,
    pub decimales: u32,
}

/// Datos que llegan del formulario, ya convertidos a tipos del dominio.
/// `categoria_id` es `None` cuando el usuario no eligió ninguna.
#[derive(Debug, Clone)]
pub struct DatosMovimiento {
    pub cuenta_id: Uuid,
    pub tipo: TipoMovimiento,
    pub monto: Decimal,
    pub fecha: NaiveDate,
    pub categoria_id: Option<Uuid>,
    pub nota: Option<String>,
}

/// Movimiento registrado. No existe ninguna operación para editarlo
/// ni eliminarlo (INV-04): solo se corrige con un `Ajuste`.
#[derive(Debug, Clone, PartialEq)]
pub struct Movimiento {
    pub id: Uuid,
    pub cuenta_id: Uuid,
    pub tipo: TipoMovimiento,
    pub monto: Decimal,
    pub fecha: NaiveDate,
    pub categoria_id: Uuid,
    pub nota: Option<String>,
}

/// Movimiento de ajuste que referencia al original.
/// `variacion_monto` = monto correcto − valor vigente del original.
#[derive(Debug, Clone, PartialEq)]
pub struct Ajuste {
    pub id: Uuid,
    pub movimiento_original_id: Uuid,
    pub cuenta_id: Uuid,
    pub tipo_original: TipoMovimiento,
    pub variacion_monto: Decimal,
}

impl Ajuste {
    /// Cuánto cambia el saldo de la cuenta por este ajuste, con signo.
    pub fn efecto_en_saldo(&self) -> Decimal {
        match self.tipo_original {
            TipoMovimiento::Ingreso => self.variacion_monto,
            TipoMovimiento::Gasto => -self.variacion_monto,
        }
    }
}

fn validar_decimales(monto: Decimal, moneda: &MonedaCuenta) -> Result<(), ErrorDominio> {
    // normalize() quita ceros de la derecha: 10,50 y 10,500 son el mismo monto.
    if monto.normalize().scale() > moneda.decimales {
        return Err(ErrorDominio::DecimalesExcedidos {
            moneda: moneda.codigo.clone(),
            maximo: moneda.decimales,
        });
    }
    Ok(())
}

/// Valida los datos y construye el movimiento. `hoy` y `id` llegan de afuera
/// para que el dominio no dependa del reloj ni de generadores de ids.
pub fn registrar_movimiento(
    id: Uuid,
    datos: DatosMovimiento,
    moneda: &MonedaCuenta,
    hoy: NaiveDate,
    categorias: &[Categoria],
) -> Result<Movimiento, ErrorDominio> {
    if datos.monto <= Decimal::ZERO {
        return Err(ErrorDominio::MontoNoPositivo);
    }
    validar_decimales(datos.monto, moneda)?;

    let categoria_id = datos.categoria_id.ok_or(ErrorDominio::CategoriaVacia)?;
    let categoria = categorias
        .iter()
        .find(|c| c.id == categoria_id)
        .ok_or(ErrorDominio::CategoriaInexistente)?;
    if categoria.tipo != datos.tipo.categoria_esperada() {
        return Err(ErrorDominio::CategoriaTipoIncorrecto);
    }

    if datos.fecha > hoy {
        return Err(ErrorDominio::FechaFutura);
    }

    let nota = datos
        .nota
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());

    Ok(Movimiento {
        id,
        cuenta_id: datos.cuenta_id,
        tipo: datos.tipo,
        monto: datos.monto,
        fecha: datos.fecha,
        categoria_id,
        nota,
    })
}

/// Monto vigente de un movimiento: el original más todos sus ajustes.
pub fn valor_vigente(original: &Movimiento, ajustes: &[Ajuste]) -> Decimal {
    original.monto
        + ajustes
            .iter()
            .filter(|a| a.movimiento_original_id == original.id)
            .map(|a| a.variacion_monto)
            .sum::<Decimal>()
}

/// Corrige un movimiento original generando un ajuste por la diferencia.
/// Los ajustes no se corrigen directamente: siempre se pasa el original.
/// Un monto correcto de 0 anula el movimiento por completo.
pub fn corregir_movimiento(
    id_ajuste: Uuid,
    original: &Movimiento,
    ajustes_previos: &[Ajuste],
    monto_correcto: Decimal,
    moneda: &MonedaCuenta,
) -> Result<Ajuste, ErrorDominio> {
    if monto_correcto < Decimal::ZERO {
        return Err(ErrorDominio::MontoCorreccionNegativo);
    }
    validar_decimales(monto_correcto, moneda)?;

    let variacion = monto_correcto - valor_vigente(original, ajustes_previos);
    if variacion.is_zero() {
        return Err(ErrorDominio::CorreccionSinCambio);
    }

    Ok(Ajuste {
        id: id_ajuste,
        movimiento_original_id: original.id,
        cuenta_id: original.cuenta_id,
        tipo_original: original.tipo,
        variacion_monto: variacion,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::categoria::catalogo_de_fabrica;

    fn d(texto: &str) -> Decimal {
        Decimal::from_str(texto).unwrap()
    }

    fn hoy() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 20).unwrap()
    }

    fn usd() -> MonedaCuenta {
        MonedaCuenta {
            codigo: "USD".to_string(),
            decimales: 2,
        }
    }

    fn categoria(nombre: &str) -> Uuid {
        catalogo_de_fabrica()
            .into_iter()
            .find(|c| c.nombre == nombre)
            .unwrap()
            .id
    }

    fn datos_gasto(monto: &str) -> DatosMovimiento {
        DatosMovimiento {
            cuenta_id: Uuid::from_u128(500),
            tipo: TipoMovimiento::Gasto,
            monto: d(monto),
            fecha: hoy(),
            categoria_id: Some(categoria("Alimentación")),
            nota: None,
        }
    }

    fn registrar(datos: DatosMovimiento) -> Result<Movimiento, ErrorDominio> {
        registrar_movimiento(
            Uuid::from_u128(900),
            datos,
            &usd(),
            hoy(),
            &catalogo_de_fabrica(),
        )
    }

    #[test]
    fn movimiento_valido_se_registra() {
        let mut datos = datos_gasto("50.00");
        datos.nota = Some("  Mercado del sábado ".to_string());
        let m = registrar(datos).unwrap();
        assert_eq!(m.monto, d("50.00"));
        assert_eq!(m.nota.as_deref(), Some("Mercado del sábado"));
    }

    #[test]
    fn monto_cero_se_rechaza_en_el_campo_monto() {
        let err = registrar(datos_gasto("0")).unwrap_err();
        assert_eq!(err, ErrorDominio::MontoNoPositivo);
        assert_eq!(err.campo(), "monto");
    }

    #[test]
    fn monto_negativo_se_rechaza() {
        let err = registrar(datos_gasto("-5.00")).unwrap_err();
        assert_eq!(err, ErrorDominio::MontoNoPositivo);
    }

    #[test]
    fn tres_decimales_en_usd_se_rechazan() {
        let err = registrar(datos_gasto("10.555")).unwrap_err();
        assert_eq!(
            err,
            ErrorDominio::DecimalesExcedidos {
                moneda: "USD".to_string(),
                maximo: 2
            }
        );
        assert_eq!(err.campo(), "monto");
    }

    #[test]
    fn ceros_a_la_derecha_no_cuentan_como_decimales() {
        assert!(registrar(datos_gasto("10.500")).is_ok());
    }

    #[test]
    fn categoria_vacia_se_rechaza_en_el_campo_categoria() {
        let mut datos = datos_gasto("10.00");
        datos.categoria_id = None;
        let err = registrar(datos).unwrap_err();
        assert_eq!(err, ErrorDominio::CategoriaVacia);
        assert_eq!(err.campo(), "categoria");
    }

    #[test]
    fn categoria_inexistente_se_rechaza() {
        let mut datos = datos_gasto("10.00");
        datos.categoria_id = Some(Uuid::from_u128(9999));
        let err = registrar(datos).unwrap_err();
        assert_eq!(err, ErrorDominio::CategoriaInexistente);
    }

    #[test]
    fn categoria_de_ingreso_en_un_gasto_se_rechaza() {
        let mut datos = datos_gasto("10.00");
        datos.categoria_id = Some(categoria("Salario"));
        let err = registrar(datos).unwrap_err();
        assert_eq!(err, ErrorDominio::CategoriaTipoIncorrecto);
        assert_eq!(err.campo(), "categoria");
    }

    #[test]
    fn fecha_futura_se_rechaza_en_el_campo_fecha() {
        let mut datos = datos_gasto("10.00");
        datos.fecha = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        let err = registrar(datos).unwrap_err();
        assert_eq!(err, ErrorDominio::FechaFutura);
        assert_eq!(err.campo(), "fecha");
    }

    #[test]
    fn fecha_de_hoy_y_fecha_pasada_se_aceptan() {
        assert!(registrar(datos_gasto("10.00")).is_ok());
        let mut pasada = datos_gasto("10.00");
        pasada.fecha = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        assert!(registrar(pasada).is_ok());
    }

    #[test]
    fn corregir_a_la_baja_genera_ajuste_positivo_para_un_gasto() {
        let original = registrar(datos_gasto("500.00")).unwrap();
        let ajuste =
            corregir_movimiento(Uuid::from_u128(901), &original, &[], d("50.00"), &usd()).unwrap();
        assert_eq!(ajuste.movimiento_original_id, original.id);
        assert_eq!(ajuste.variacion_monto, d("-450.00"));
        assert_eq!(ajuste.efecto_en_saldo(), d("450.00"));
    }

    #[test]
    fn monto_correcto_cero_compensa_por_completo() {
        let original = registrar(datos_gasto("80.00")).unwrap();
        let ajuste =
            corregir_movimiento(Uuid::from_u128(901), &original, &[], d("0"), &usd()).unwrap();
        assert_eq!(ajuste.efecto_en_saldo(), d("80.00"));
    }

    #[test]
    fn segunda_correccion_se_calcula_sobre_el_valor_vigente() {
        let original = registrar(datos_gasto("500.00")).unwrap();
        let primero =
            corregir_movimiento(Uuid::from_u128(901), &original, &[], d("50.00"), &usd()).unwrap();
        let segundo = corregir_movimiento(
            Uuid::from_u128(902),
            &original,
            std::slice::from_ref(&primero),
            d("60.00"),
            &usd(),
        )
        .unwrap();
        assert_eq!(segundo.variacion_monto, d("10.00"));
        assert_eq!(valor_vigente(&original, &[primero, segundo]), d("60.00"));
    }

    #[test]
    fn correccion_negativa_o_sin_cambio_o_con_decimales_de_mas_se_rechaza() {
        let original = registrar(datos_gasto("50.00")).unwrap();
        let id = Uuid::from_u128(901);

        let negativa = corregir_movimiento(id, &original, &[], d("-1"), &usd()).unwrap_err();
        assert_eq!(negativa, ErrorDominio::MontoCorreccionNegativo);
        assert_eq!(negativa.campo(), "monto_correcto");

        let igual = corregir_movimiento(id, &original, &[], d("50.00"), &usd()).unwrap_err();
        assert_eq!(igual, ErrorDominio::CorreccionSinCambio);

        let decimales = corregir_movimiento(id, &original, &[], d("10.555"), &usd()).unwrap_err();
        assert!(matches!(decimales, ErrorDominio::DecimalesExcedidos { .. }));
    }
}
