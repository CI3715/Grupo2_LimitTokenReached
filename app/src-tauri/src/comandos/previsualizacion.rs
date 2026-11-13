use contrato::{ParteGastoDTO, RegistrarGastoRequestDTO};

#[tauri::command(rename_all = "snake_case")]
pub fn previsualizar_division(
    datos: RegistrarGastoRequestDTO,
) -> Result<Vec<ParteGastoDTO>, String> {
    aplicacion::casos_uso::previsualizar_division::ejecutar(datos)
}
