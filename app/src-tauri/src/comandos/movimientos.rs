use aplicacion::puertos::{Reloj, RepositorioCuentas, RepositorioMonedas, RepositorioMovimientos};
use contrato::error_validacion::ErrorValidacionDTO;
use contrato::movimiento::{CorregirMovimientoRequest, MovimientoDTO, RegistrarMovimientoRequest};
use tauri::State;

/// Comando Tauri para registrar un nuevo movimiento (ingreso o gasto).
/// Recibe el formulario desde Next.js y delega al caso de uso correspondiente.
#[tauri::command]
pub async fn registrar_movimiento(
    datos: RegistrarMovimientoRequest,
    repo_movimientos: State<'_, Box<dyn RepositorioMovimientos>>,
    repo_cuentas: State<'_, Box<dyn RepositorioCuentas>>,
    repo_monedas: State<'_, Box<dyn RepositorioMonedas>>,
    reloj: State<'_, Box<dyn Reloj>>,
) -> Result<MovimientoDTO, ErrorValidacionDTO> {
    aplicacion::casos_uso::registrar_movimiento::ejecutar(
        datos,
        repo_movimientos.inner().as_ref(),
        repo_cuentas.inner().as_ref(),
        repo_monedas.inner().as_ref(),
        reloj.inner().as_ref(),
    )
    .await
}

/// Se invoca cuando sobre un movimiento se elige "Corregir" y se indica el monto correcto.
/// Registra el ajuste; el original queda marcado como corregido y el saldo muestra el valor correcto
#[tauri::command]
pub async fn corregir_movimiento(
    datos: CorregirMovimientoRequest,
    repo_movimientos: State<'_, Box<dyn RepositorioMovimientos>>,
    repo_cuentas: State<'_, Box<dyn RepositorioCuentas>>,
    repo_monedas: State<'_, Box<dyn RepositorioMonedas>>,
) -> Result<MovimientoDTO, ErrorValidacionDTO> {
    aplicacion::casos_uso::corregir_movimiento::ejecutar(
        datos,
        repo_movimientos.inner().as_ref(),
        repo_cuentas.inner().as_ref(),
        repo_monedas.inner().as_ref(),
    )
    .await
}

#[tauri::command]
pub async fn listar_historial_movimientos(
    cuenta_id: String,
    repo_movimientos: State<'_, Box<dyn RepositorioMovimientos>>,
) -> Result<Vec<MovimientoDTO>, ErrorValidacionDTO> {
    aplicacion::casos_uso::obtener_historial_movimientos::ejecutar(
        cuenta_id,
        repo_movimientos.inner().as_ref(),
    )
    .await
}
