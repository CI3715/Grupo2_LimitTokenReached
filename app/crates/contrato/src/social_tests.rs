use std::collections::HashMap;

use super::{
    DetalleGrupoDTO, GastoCreadoDTO, GastoDTO, MiembroGrupoDTO, ParteGastoDTO,
};

#[test]
fn gasto_creado_conserva_montos_y_marca_de_residuo() {
    let esperado = GastoCreadoDTO {
        id: "gasto-1".to_string(),
        partes: vec![
            ParteGastoDTO {
                usuario: "ana".to_string(),
                monto: "33.34".to_string(),
                residuo: true,
            },
            ParteGastoDTO {
                usuario: "beto".to_string(),
                monto: "33.33".to_string(),
                residuo: false,
            },
        ],
    };

    let json = serde_json::to_string(&esperado).unwrap();
    let recibido: GastoCreadoDTO = serde_json::from_str(&json).unwrap();

    assert_eq!(recibido, esperado);
}

#[test]
fn detalle_de_grupo_conserva_balances_como_texto_decimal() {
    let mut balances = HashMap::new();
    balances.insert("ana".to_string(), "36.66".to_string());
    balances.insert("beto".to_string(), "-48.33".to_string());

    let esperado = DetalleGrupoDTO {
        id: "grupo-1".to_string(),
        nombre: "Viaje".to_string(),
        descripcion: None,
        moneda: "USD".to_string(),
        actividad: None,
        miembros: vec![MiembroGrupoDTO {
            nombre_usuario: "ana".to_string(),
            nombre_visible: "Ana".to_string(),
            rol: "miembro".to_string(),
        }],
        gastos: vec![GastoDTO {
            id: "gasto-1".to_string(),
            concepto: "Cena".to_string(),
            monto: "100.00".to_string(),
            fecha: "2026-11-12".to_string(),
            pagador: "ana".to_string(),
            regla: "iguales".to_string(),
            partes: vec![],
        }],
        balances,
        partes: vec![],
    };

    let json = serde_json::to_string(&esperado).unwrap();
    let recibido: DetalleGrupoDTO = serde_json::from_str(&json).unwrap();

    assert_eq!(recibido.balances["ana"], "36.66");
    assert_eq!(recibido.balances["beto"], "-48.33");
}

#[test]
fn los_dto_social_rechazan_campos_desconocidos() {
    let json = r#"{
        "id": "gasto-1",
        "partes": [],
        "campo_no_soportado": true
    }"#;

    assert!(serde_json::from_str::<GastoCreadoDTO>(json).is_err());
}
