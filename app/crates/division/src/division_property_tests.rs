#[cfg(test)]
mod tests {
    use crate::division::*;
    use proptest::prelude::*;
    use rust_decimal::prelude::*;

    // INV-07: Property-Based Testing
    // La suma de las partes SIEMPRE es exactamente el monto total

    proptest! {
        #[test]
        fn prop_inv07_partes_iguales_suman_el_total(
            monto_centavos in 1i64..10_000_000i64, // 0.01 a 99,999.99
            num_participantes in 1usize..20usize,
        ) {
            let monto = Decimal::new(monto_centavos, 2);
            let participantes: Vec<String> = (0..num_participantes)
                .map(|i| format!("user_{}", i))
                .collect();

            let regla = ReglaDivision::PartesIguales { participantes };
            let partes = calcular_division(monto, &regla).unwrap();
            let suma: Decimal = partes.iter().map(|p| p.monto).sum();

            prop_assert_eq!(
                suma, monto,
                "INV-07: las partes deben sumar exactamente el total (monto={}, participantes={})",
                monto, num_participantes
            );
        }

        #[test]
        fn prop_inv07_porcentajes_suman_el_total(
            monto_centavos in 1i64..10_000_000i64,
            num_participantes in 1usize..10usize,
        ) {
            let monto = Decimal::new(monto_centavos, 2);
            
            // Generar porcentajes que sumen exactamente 100
            let mut porcentajes = Vec::new();
            let mut restante = 10000i64; // 100.00% en centésimas de porcentaje
            
            for i in 0..num_participantes {
                if i == num_participantes - 1 {
                    // El último recibe todo lo que queda
                    porcentajes.push((format!("user_{}", i), Decimal::new(restante, 2)));
                } else {
                    // Generar un porcentaje aleatorio entre 1 y lo que queda
                    let max_porcentaje = restante.min(9900); // máximo 99%
                    let porcentaje = if max_porcentaje > 0 {
                        (proptest::test_runner::Config::default().clone_rng().gen_range(1..=max_porcentaje)) as i64
                    } else {
                        1
                    };
                    porcentajes.push((format!("user_{}", i), Decimal::new(porcentaje, 2)));
                    restante -= porcentaje;
                }
            }

            let regla = ReglaDivision::Porcentajes { porcentajes };
            let partes = calcular_division(monto, &regla).unwrap();
            let suma: Decimal = partes.iter().map(|p| p.monto).sum();

            prop_assert_eq!(
                suma, monto,
                "INV-07: las partes por porcentajes deben sumar exactamente el total"
            );
        }

        #[test]
        fn prop_inv07_residuo_se_reparte_correctamente(
            monto_centavos in 1i64..10_000_000i64,
            num_participantes in 2usize..10usize,
        ) {
            let monto = Decimal::new(monto_centavos, 2);
            let participantes: Vec<String> = (0..num_participantes)
                .map(|i| format!("user_{}", i))
                .collect();

            let regla = ReglaDivision::PartesIguales { participantes };
            let partes = calcular_division(monto, &regla).unwrap();

            // Verificar que el residuo se reparte en orden alfabético
            let partes_con_redondeo: Vec<_> = partes.iter()
                .filter(|p| p.tiene_redondeo)
                .collect();

            // Verificar que los usernames con redondeo están ordenados alfabéticamente
            for i in 1..partes_con_redondeo.len() {
                prop_assert!(
                    partes_con_redondeo[i-1].username < partes_con_redondeo[i].username,
                    "El residuo debe repartirse en orden alfabético"
                );
            }
        }

        #[test]
        fn prop_inv08_suma_de_balances_es_cero(
            num_gastos in 1usize..5usize,
        ) {
            // Generar gastos aleatorios
            let mut gastos = Vec::new();
            let mut todos_los_users = std::collections::HashSet::new();

            for _ in 0..num_gastos {
                let monto_centavos = proptest::test_runner::Config::default().clone_rng().gen_range(100..10000);
                let monto = Decimal::new(monto_centavos, 2);
                let num_participantes = proptest::test_runner::Config::default().clone_rng().gen_range(1usize..5usize);
                
                let participantes: Vec<String> = (0..num_participantes)
                    .map(|i| {
                        let username = format!("user_{}", i);
                        todos_los_users.insert(username.clone());
                        username
                    })
                    .collect();

                let pagador = participantes[0].clone();
                let regla = ReglaDivision::PartesIguales { participantes };
                let partes = calcular_division(monto, &regla).unwrap();

                gastos.push((pagador, monto, partes));
            }

            // Calcular balances
            let mut balances = Vec::new();
            for username in todos_los_users {
                let balance = calcular_balance(&username, &gastos);
                balances.push((username, balance));
            }

            // INV-08: la suma de balances debe ser cero
            let suma_balances: Decimal = balances.iter().map(|(_, b)| *b).sum();
            prop_assert_eq!(
                suma_balances,
                Decimal::new(0, 2),
                "INV-08: la suma de balances debe ser exactamente cero"
            );
        }
    }

    // Pruebas adicionales con datos específicos

    #[test]
    fn prop_monto_muy_pequeno_se_divide_correctamente() {
        let monto = Decimal::new(1, 2); // 0.01 (el monto más pequeño posible)
        let participantes = vec!["ana".to_string(), "beto".to_string()];
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(suma, monto, "INV-07: incluso con 0.01, las partes suman el total");
    }

    #[test]
    fn prop_monto_muy_grande_se_divide_con_precision() {
        let monto = Decimal::new(99999999, 2); // 999,999.99
        let participantes = vec![
            "user_0".to_string(),
            "user_1".to_string(),
            "user_2".to_string(),
            "user_3".to_string(),
            "user_4".to_string(),
        ];
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(suma, monto, "INV-07: con montos grandes, las partes suman el total");
    }

    #[test]
    fn prop_muchos_participantes_se_divide_correctamente() {
        let monto = Decimal::new(100000, 2); // 1,000.00
        let participantes: Vec<String> = (0..50).map(|i| format!("user_{:02}", i)).collect();
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(suma, monto, "INV-07: con 50 participantes, las partes suman el total");
    }
}