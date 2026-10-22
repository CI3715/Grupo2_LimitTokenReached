
use contrato::movimiento::{CorregirMovimientoRequest, MovimientoDTO};
use contrato::error_validacion::ErrorValidacionDTO;
use crate::puertos::{RepositorioMovimientos, RepositorioCuentas, RepositorioMonedas};
use dominio::movimiento::{corregir_movimiento as corregir_movimiento_dominio, MonedaCuenta,};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

/// Orquesta el flujo de corrección de un movimiento garantizando la inmutabilidad (RF-LIB-06).
pub async fn ejecutar(
    datos: CorregirMovimientoRequest,
    repo_movimientos: &dyn RepositorioMovimientos,
    repo_cuentas: &dyn RepositorioCuentas,
    repo_monedas: &dyn RepositorioMonedas,
) -> Result<MovimientoDTO, ErrorValidacionDTO> { // Idealmente retornar ErrorValidacionDTO

    // 1. Parsear los datos de entrada que vienen del frontend
    let movimiento_id = Uuid::from_str(&datos.movimiento_id)
    .map_err(|_| ErrorValidacionDTO {
        campo: "movimiento_id".to_string(),
        mensaje: "ID de movimiento original inválido".to_string(),
    })?;

    let monto_correcto = Decimal::from_str(&datos.monto_correcto)
    .map_err(|_| ErrorValidacionDTO {
        campo: "monto_correcto".to_string(),
        mensaje: "El monto correcto debe ser un número decimal válido".to_string(),
    })?;

    // 2. Obtener el movimiento original desde la base de datos a través del puerto.
    // Recuerda: Siempre se corrige el movimiento original.
    let movimiento_original = repo_movimientos
    .obtener_movimiento_por_id(&movimiento_id)
    .map_err(|mensaje| ErrorValidacionDTO {
        campo: "general".to_string(),
        mensaje,
    })?;
    
    // 3. Obtener todos los ajustes previos que se le hayan aplicado a este movimiento.
    // Esto es necesario porque un movimiento puede corregirse más de una vez[cite: 1].
    let ajustes_previos = repo_movimientos
    .obtener_ajustes_de_movimiento(&movimiento_id)
    .map_err(|mensaje| ErrorValidacionDTO {
        campo: "general".to_string(),
        mensaje,
    })?;

    let cuenta = repo_cuentas
    .obtener_cuenta_por_id(&movimiento_original.cuenta_id.to_string())
    .map_err(|mensaje| ErrorValidacionDTO {
        campo: "cuenta_id".to_string(),
        mensaje,
    })?;

    let monedas = repo_monedas
    .obtener_monedas()
    .map_err(|mensaje| ErrorValidacionDTO {
        campo: "moneda_codigo".to_string(),
        mensaje,
    })?;

    let moneda = monedas
    .iter()
    .find(|moneda| moneda.codigo == cuenta.moneda_codigo)
    .ok_or_else(|| ErrorValidacionDTO {
        campo: "moneda_codigo".to_string(),
        mensaje: "La moneda de la cuenta no existe en el catálogo".to_string(),
    })?;

    let moneda_cuenta = MonedaCuenta {
        codigo: moneda.codigo.clone(),
        decimales: moneda.decimales,
    };

    // 5. Generar el identificador único para este nuevo ajuste compensatorio
    let id_ajuste = Uuid::new_v4();

    // 6. INVOCAR LA LÓGICA PURA DE DOMINIO
    // Tu compañera se encarga aquí de calcular la diferencia matemática y validar que 
    // el monto no sea negativo, o devolver 0 si el movimiento se anula por completo[cite: 1].
    let nuevo_ajuste = corregir_movimiento_dominio(
    id_ajuste,
    &movimiento_original,
    &ajustes_previos,
    monto_correcto,
    &moneda_cuenta,
    )
    .map_err(|err| ErrorValidacionDTO {
        campo: "monto_correcto".to_string(),
        mensaje: format!("Error de dominio al corregir: {:?}", err),
    })?;
    
    // 7. Delegar al puerto la persistencia (esto insertará el evento MovimientoCorregido 
    // en SQLite y actualizará el saldo, pero sin borrar ni reescribir el original[cite: 1]).
    repo_movimientos
    .guardar_ajuste(&nuevo_ajuste)
    .map_err(|mensaje| ErrorValidacionDTO {
        campo: "general".to_string(),
        mensaje,
    })
}