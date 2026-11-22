use aplicacion::puertos::{
    ClienteSocial, RepositorioColaGastos, RepositorioCuentas, RepositorioMovimientos,
    RepositorioSincronizacion,
};
use contrato::{
    CuentaGrupoSeleccionDTO, EstadoSincronizacionDTO, GastoColaDTO,
    ParteGrupoPendienteDTO,
};
use tauri::{AppHandle, State};

use super::sesion::{borrar_token, obtener_token};

fn manejar_error(app: &AppHandle, error: aplicacion::puertos::ErrorOperacion) -> String {
    if error.status == Some(401) {
        let _ = borrar_token(app);
        return "Tu sesión venció, inicia sesión de nuevo.".to_string();
    }
    error.mensaje
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sincronizar_grupos(
    app: AppHandle,
    cliente: State<'_, Box<dyn ClienteSocial>>,
    repo_sincronizacion: State<'_, Box<dyn RepositorioSincronizacion>>,
    repo_movimientos: State<'_, Box<dyn RepositorioMovimientos>>,
    repo_cuentas: State<'_, Box<dyn RepositorioCuentas>>,
    repo_cola: State<'_, Box<dyn RepositorioColaGastos>>,
) -> Result<u64, String> {
    let token = obtener_token(&app)?;

    aplicacion::casos_uso::procesar_cola_gastos::ejecutar(
        cliente.inner().as_ref(),
        token.clone(),
        repo_cola.inner().as_ref(),
    )
    .await
    .map_err(|error| manejar_error(&app, error))?;

    aplicacion::casos_uso::sincronizar_cambios::ejecutar(
        cliente.inner().as_ref(),
        token,
        repo_sincronizacion.inner().as_ref(),
        repo_movimientos.inner().as_ref(),
        repo_cuentas.inner().as_ref(),
    )
    .await
    .map_err(|error| manejar_error(&app, error))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn elegir_cuenta_grupo(
    seleccion: CuentaGrupoSeleccionDTO,
    moneda_grupo: String,
    repo_sincronizacion: State<'_, Box<dyn RepositorioSincronizacion>>,
    repo_movimientos: State<'_, Box<dyn RepositorioMovimientos>>,
    repo_cuentas: State<'_, Box<dyn RepositorioCuentas>>,
) -> Result<usize, String> {
    aplicacion::casos_uso::elegir_cuenta_grupo::ejecutar(
        seleccion,
        &moneda_grupo,
        repo_sincronizacion.inner().as_ref(),
        repo_movimientos.inner().as_ref(),
        repo_cuentas.inner().as_ref(),
    )
}

#[tauri::command(rename_all = "snake_case")]
pub async fn listar_partes_grupo_pendientes(
    repo_movimientos: State<'_, Box<dyn RepositorioMovimientos>>,
) -> Result<Vec<ParteGrupoPendienteDTO>, String> {
    repo_movimientos
        .inner()
        .as_ref()
        .listar_partes_grupo_pendientes()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn listar_cola_gastos_grupo(
    repo_cola: State<'_, Box<dyn RepositorioColaGastos>>,
) -> Result<Vec<GastoColaDTO>, String> {
    repo_cola.inner().as_ref().listar_estados_en_orden()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn estado_sincronizacion(
    repo_sincronizacion: State<'_, Box<dyn RepositorioSincronizacion>>,
) -> Result<EstadoSincronizacionDTO, String> {
    Ok(EstadoSincronizacionDTO {
        cursor: repo_sincronizacion.inner().as_ref().obtener_cursor()?,
        ultima_sincronizacion: repo_sincronizacion
            .inner()
            .as_ref()
            .obtener_ultima_sincronizacion()?,
        sesion_vencida: false,
    })
}
