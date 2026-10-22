use rust_decimal::Decimal;

use crate::movimiento::{Ajuste, Movimiento, TipoMovimiento};

/// Cómo se compone el saldo de una cuenta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposicionSaldo {
    pub saldo_inicial: Decimal,
    pub total_ingresos: Decimal,
    pub total_gastos: Decimal,
    /// Suma de los efectos en el saldo de todos los ajustes (con signo).
    pub total_ajustes: Decimal,
    pub saldo_actual: Decimal,
}

/// saldo actual = saldo inicial + ingresos − gastos ± ajustes (INV-05).
pub fn calcular_composicion(
    saldo_inicial: Decimal,
    movimientos: &[Movimiento],
    ajustes: &[Ajuste],
) -> ComposicionSaldo {
    let total = |tipo: TipoMovimiento| -> Decimal {
        movimientos
            .iter()
            .filter(|m| m.tipo == tipo)
            .map(|m| m.monto)
            .sum()
    };
    let total_ingresos = total(TipoMovimiento::Ingreso);
    let total_gastos = total(TipoMovimiento::Gasto);
    let total_ajustes: Decimal = ajustes.iter().map(Ajuste::efecto_en_saldo).sum();

    ComposicionSaldo {
        saldo_inicial,
        total_ingresos,
        total_gastos,
        total_ajustes,
        saldo_actual: saldo_inicial + total_ingresos - total_gastos + total_ajustes,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::NaiveDate;
    use uuid::Uuid;

    use super::*;
    use crate::categoria::catalogo_de_fabrica;
    use crate::movimiento::{
        DatosMovimiento, MonedaCuenta, corregir_movimiento, registrar_movimiento,
    };

    fn d(texto: &str) -> Decimal {
        Decimal::from_str(texto).unwrap()
    }

    fn movimiento(id: u128, tipo: TipoMovimiento, monto: &str, categoria: &str) -> Movimiento {
        let categorias = catalogo_de_fabrica();
        let categoria_id = categorias
            .iter()
            .find(|c| c.nombre == categoria)
            .unwrap()
            .id;
        let hoy = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
        let moneda = MonedaCuenta {
            codigo: "USD".to_string(),
            decimales: 2,
        };
        registrar_movimiento(
            Uuid::from_u128(id),
            DatosMovimiento {
                cuenta_id: Uuid::from_u128(500),
                tipo,
                monto: d(monto),
                fecha: hoy,
                categoria_id: Some(categoria_id),
                nota: None,
            },
            &moneda,
            hoy,
            &categorias,
        )
        .unwrap()
    }

    #[test]
    fn cuenta_sin_movimientos_tiene_el_saldo_inicial() {
        let c = calcular_composicion(d("1000.00"), &[], &[]);
        assert_eq!(c.saldo_actual, d("1000.00"));
    }

    #[test]
    fn ingreso_sube_y_gasto_baja_exactamente_en_el_monto() {
        let ingreso = movimiento(1, TipoMovimiento::Ingreso, "800.00", "Salario");
        let gasto = movimiento(2, TipoMovimiento::Gasto, "50.00", "Alimentación");
        let c = calcular_composicion(d("1000.00"), &[ingreso, gasto], &[]);
        assert_eq!(c.saldo_actual, d("1750.00"));
    }

    #[test]
    fn ejemplo_de_la_seccion_2_2_de_la_spec() {
        let ingreso = movimiento(1, TipoMovimiento::Ingreso, "800.00", "Salario");
        let gasto = movimiento(2, TipoMovimiento::Gasto, "500.00", "Alimentación");

        let antes = calcular_composicion(d("1000.00"), &[ingreso.clone(), gasto.clone()], &[]);
        assert_eq!(antes.saldo_actual, d("1300.00"));

        let moneda = MonedaCuenta {
            codigo: "USD".to_string(),
            decimales: 2,
        };
        let ajuste =
            corregir_movimiento(Uuid::from_u128(3), &gasto, &[], d("50.00"), &moneda).unwrap();
        assert_eq!(ajuste.efecto_en_saldo(), d("450.00"));

        let despues = calcular_composicion(d("1000.00"), &[ingreso, gasto], &[ajuste]);
        assert_eq!(despues.total_ingresos, d("800.00"));
        assert_eq!(despues.total_gastos, d("500.00"));
        assert_eq!(despues.total_ajustes, d("450.00"));
        assert_eq!(despues.saldo_actual, d("1750.00"));
    }

    #[test]
    fn el_saldo_actual_siempre_es_la_suma_de_sus_partes() {
        let ingreso = movimiento(1, TipoMovimiento::Ingreso, "10.10", "Ventas");
        let gasto = movimiento(2, TipoMovimiento::Gasto, "0.30", "Salud");
        let c = calcular_composicion(d("0.20"), &[ingreso, gasto], &[]);
        assert_eq!(
            c.saldo_actual,
            c.saldo_inicial + c.total_ingresos - c.total_gastos + c.total_ajustes
        );
        assert_eq!(c.saldo_actual, d("10.00"));
    }
}
