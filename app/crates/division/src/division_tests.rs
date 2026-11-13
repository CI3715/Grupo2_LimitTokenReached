#[cfg(test)]
mod tests {
    use crate::division::*;
    use rust_decimal::prelude::*;

    // INV-07: La suma de partes es exactamente el total
    #[test]
    fn inv07_partes_iguales_suman_exactamente_el_total() {
        let monto = Decimal::new(10000, 2); // 100.00
        let participantes = vec!["ana".to_string(), "beto".to_string(), "carla".to_string()];
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(
            suma, monto,
            "INV-07: las partes deben sumar exactamente el total"
        );
    }

    #[test]
    fn inv07_porcentajes_suman_exactamente_el_total() {
        let monto = Decimal::new(1000, 2); // 10.00
        let porcentajes = vec![
            ("ana".to_string(), Decimal::new(3333, 2)),
            ("beto".to_string(), Decimal::new(3333, 2)),
            ("carla".to_string(), Decimal::new(3334, 2)),
        ];
        let regla = ReglaDivision::Porcentajes { porcentajes };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(
            suma, monto,
            "INV-07: las partes deben sumar exactamente el total"
        );
    }

    #[test]
    fn inv07_montos_fijos_suman_exactamente_el_total() {
        let monto = Decimal::new(12000, 2); // 120.00
        let montos = vec![
            ("ana".to_string(), Decimal::new(5000, 2)),
            ("beto".to_string(), Decimal::new(4000, 2)),
            ("carla".to_string(), Decimal::new(3000, 2)),
        ];
        let regla = ReglaDivision::MontosFijos { montos };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(
            suma, monto,
            "INV-07: las partes deben sumar exactamente el total"
        );
    }

    // Ejemplo exacto de la sección 2.4 de la especificación

    #[test]
    fn ejemplo_spec_2_4_partes_iguales_100_entre_3() {
        let monto = Decimal::new(10000, 2); // 100.00
        let participantes = vec!["carla".to_string(), "ana".to_string(), "beto".to_string()];
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();

        // ana: 33.34 (con redondeo), beto: 33.33, carla: 33.33
        let ana = partes.iter().find(|p| p.username == "ana").unwrap();
        let beto = partes.iter().find(|p| p.username == "beto").unwrap();
        let carla = partes.iter().find(|p| p.username == "carla").unwrap();

        assert_eq!(ana.monto, Decimal::new(3334, 2));
        assert!(ana.tiene_redondeo);
        assert_eq!(beto.monto, Decimal::new(3333, 2));
        assert!(!beto.tiene_redondeo);
        assert_eq!(carla.monto, Decimal::new(3333, 2));
        assert!(!carla.tiene_redondeo);
    }

    #[test]
    fn ejemplo_spec_2_4_porcentajes_10_entre_3() {
        let monto = Decimal::new(1000, 2); // 10.00
        let porcentajes = vec![
            ("carla".to_string(), Decimal::new(3334, 2)),
            ("ana".to_string(), Decimal::new(3333, 2)),
            ("beto".to_string(), Decimal::new(3333, 2)),
        ];
        let regla = ReglaDivision::Porcentajes { porcentajes };

        let partes = calcular_division(monto, &regla).unwrap();

        let ana = partes.iter().find(|p| p.username == "ana").unwrap();
        let beto = partes.iter().find(|p| p.username == "beto").unwrap();
        let carla = partes.iter().find(|p| p.username == "carla").unwrap();

        assert_eq!(ana.monto, Decimal::new(334, 2)); // 3.34
        assert!(ana.tiene_redondeo);
        assert_eq!(beto.monto, Decimal::new(333, 2)); // 3.33
        assert_eq!(carla.monto, Decimal::new(333, 2)); // 3.33
    }

    // INV-08: La suma de balances es exactamente cero
    #[test]
    fn inv08_suma_de_balances_es_cero_ejemplo_spec_2_5() {
        // ana paga 100.00 en partes iguales
        // beto paga 60.00 por porcentajes (ana 50%, beto 25%, carla 25%)
        let gastos = vec![
            (
                "ana".to_string(),
                Decimal::new(10000, 2),
                vec![
                    ParteCalculada {
                        username: "ana".to_string(),
                        monto: Decimal::new(3334, 2),
                        tiene_redondeo: true,
                    },
                    ParteCalculada {
                        username: "beto".to_string(),
                        monto: Decimal::new(3333, 2),
                        tiene_redondeo: false,
                    },
                    ParteCalculada {
                        username: "carla".to_string(),
                        monto: Decimal::new(3333, 2),
                        tiene_redondeo: false,
                    },
                ],
            ),
            (
                "beto".to_string(),
                Decimal::new(6000, 2),
                vec![
                    ParteCalculada {
                        username: "ana".to_string(),
                        monto: Decimal::new(3000, 2),
                        tiene_redondeo: false,
                    },
                    ParteCalculada {
                        username: "beto".to_string(),
                        monto: Decimal::new(1500, 2),
                        tiene_redondeo: false,
                    },
                    ParteCalculada {
                        username: "carla".to_string(),
                        monto: Decimal::new(1500, 2),
                        tiene_redondeo: false,
                    },
                ],
            ),
        ];

        let balance_ana = calcular_balance("ana", &gastos);
        let balance_beto = calcular_balance("beto", &gastos);
        let balance_carla = calcular_balance("carla", &gastos);

        let balances = vec![
            ("ana".to_string(), balance_ana),
            ("beto".to_string(), balance_beto),
            ("carla".to_string(), balance_carla),
        ];

        assert!(
            verificar_suma_balances_cero(&balances),
            "INV-08: la suma de balances debe ser cero"
        );

        // Valores esperados del ejemplo de la spec
        assert_eq!(balance_ana, Decimal::new(3666, 2), "ana: +36.66");
        assert_eq!(balance_beto, Decimal::new(1167, 2), "beto: +11.67");
        assert_eq!(balance_carla, Decimal::new(-4833, 2), "carla: -48.33");
    }

    // Validaciones

    #[test]
    fn porcentajes_que_no_suman_100_se_rechazan() {
        let monto = Decimal::new(1000, 2);
        let porcentajes = vec![
            ("ana".to_string(), Decimal::new(3000, 2)),
            ("beto".to_string(), Decimal::new(3000, 2)),
            ("carla".to_string(), Decimal::new(3000, 2)),
        ];
        let regla = ReglaDivision::Porcentajes { porcentajes };

        let resultado = calcular_division(monto, &regla);
        assert!(
            resultado.is_err(),
            "Porcentajes que suman 90 deben rechazarse"
        );
    }

    #[test]
    fn montos_fijos_que_no_suman_el_total_se_rechazan() {
        let monto = Decimal::new(12000, 2);
        let montos = vec![
            ("ana".to_string(), Decimal::new(5000, 2)),
            ("beto".to_string(), Decimal::new(4000, 2)),
            ("carla".to_string(), Decimal::new(2000, 2)),
        ];
        let regla = ReglaDivision::MontosFijos { montos };

        let resultado = calcular_division(monto, &regla);
        assert!(
            resultado.is_err(),
            "Montos que no suman el total deben rechazarse"
        );
    }

    #[test]
    fn sin_participantes_se_rechaza() {
        let monto = Decimal::new(10000, 2);
        let regla = ReglaDivision::PartesIguales {
            participantes: vec![],
        };

        let resultado = calcular_division(monto, &regla);
        assert!(resultado.is_err(), "Sin participantes debe rechazarse");
    }

    // Casos extremos

    #[test]
    fn un_solo_participante_recibe_todo_el_monto() {
        let monto = Decimal::new(5000, 2);
        let participantes = vec!["ana".to_string()];
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();
        assert_eq!(partes.len(), 1);
        assert_eq!(partes[0].monto, monto);
    }

    #[test]
    fn truncamiento_y_residuo_evitan_perder_centavos() {
        let monto = Decimal::new(5, 2); // 0.05
        let participantes = vec!["beto".to_string(), "ana".to_string()];
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();
        let ana = partes.iter().find(|p| p.username == "ana").unwrap();
        let beto = partes.iter().find(|p| p.username == "beto").unwrap();

        assert_eq!(ana.monto, Decimal::new(3, 2));
        assert!(ana.tiene_redondeo);
        assert_eq!(beto.monto, Decimal::new(2, 2));
        assert!(!beto.tiene_redondeo);
    }

    #[test]
    fn porcentajes_reparten_residuo_despues_de_truncar() {
        let monto = Decimal::new(5, 2); // 0.05
        let porcentajes = vec![
            ("beto".to_string(), Decimal::new(5000, 2)),
            ("ana".to_string(), Decimal::new(5000, 2)),
        ];
        let regla = ReglaDivision::Porcentajes { porcentajes };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(suma, monto);
        assert_eq!(
            partes.iter().find(|p| p.username == "ana").unwrap().monto,
            Decimal::new(3, 2)
        );
    }

    #[test]
    fn porcentajes_con_suma_inexacta_se_rechazan() {
        let regla = ReglaDivision::Porcentajes {
            porcentajes: vec![
                ("ana".to_string(), Decimal::new(9998, 2)),
                ("beto".to_string(), Decimal::new(1, 2)),
            ],
        };

        assert!(calcular_division(Decimal::ONE, &regla).is_err());
    }

    #[test]
    fn participantes_repetidos_se_rechazan() {
        let regla = ReglaDivision::PartesIguales {
            participantes: vec!["ana".to_string(), "ana".to_string()],
        };

        assert!(calcular_division(Decimal::ONE, &regla).is_err());
    }

    #[test]
    fn monto_grande_se_divide_con_precision() {
        let monto = Decimal::new(99999999, 2); // 999,999.99
        let participantes = vec!["ana".to_string(), "beto".to_string(), "carla".to_string()];
        let regla = ReglaDivision::PartesIguales { participantes };

        let partes = calcular_division(monto, &regla).unwrap();
        let suma: Decimal = partes.iter().map(|p| p.monto).sum();

        assert_eq!(
            suma, monto,
            "INV-07: incluso con montos grandes, las partes suman el total"
        );
    }
}
