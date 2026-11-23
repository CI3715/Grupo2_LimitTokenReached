use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use aplicacion::puertos::{
    ClienteSocial, RepositorioColaGastos, RepositorioCuentas, RepositorioMovimientos,
    RepositorioSincronizacion,
};
use contrato::{
    CuentaGrupoSeleccionDTO, EstadoSincronizacionDTO, GastoColaDTO, ParteGrupoPendienteDTO,
};
use tauri::{AppHandle, Emitter, Manager, State};

use super::sesion::{borrar_token, obtener_token};

static SINCRONIZANDO: AtomicBool = AtomicBool::new(false);

struct GuardiaSincronizacion;

impl Drop for GuardiaSincronizacion {
    fn drop(&mut self) {
        SINCRONIZANDO.store(false, Ordering::SeqCst);
    }
}

fn manejar_error(app: &AppHandle, error: aplicacion::puertos::ErrorOperacion) -> String {
    if error.status == Some(401) {
        let _ = borrar_token(app);
        return "Tu sesión venció, inicia sesión de nuevo.".to_string();
    }
    error.mensaje
}

async fn ejecutar_sincronizacion(app: &AppHandle) -> Result<u64, String> {
    if SINCRONIZANDO
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Ya hay una sincronización en curso.".to_string());
    }
    let _guardia = GuardiaSincronizacion;

    let token = obtener_token(app)?;

    let cliente = app.state::<Box<dyn ClienteSocial>>();
    let repo_sincronizacion = app.state::<Box<dyn RepositorioSincronizacion>>();
    let repo_cuentas = app.state::<Box<dyn RepositorioCuentas>>();
    let repo_cola = app.state::<Box<dyn RepositorioColaGastos>>();

    let resultado_cola = aplicacion::casos_uso::procesar_cola_gastos::ejecutar(
        cliente.inner().as_ref(),
        token.clone(),
        repo_cola.inner().as_ref(),
    )
    .await;

    if let Err(error) = &resultado_cola
        && error.status == Some(401)
    {
        let _ = repo_sincronizacion
            .inner()
            .as_ref()
            .guardar_sesion_vencida(true);
        return Err(manejar_error(app, error.clone()));
    }
    resultado_cola.map_err(|error| manejar_error(app, error))?;

    let resultado_cambios = aplicacion::casos_uso::sincronizar_cambios::ejecutar(
        cliente.inner().as_ref(),
        token,
        repo_sincronizacion.inner().as_ref(),
        repo_cuentas.inner().as_ref(),
    )
    .await;

    match resultado_cambios {
        Ok(cursor) => {
            let _ = repo_sincronizacion
                .inner()
                .as_ref()
                .guardar_sesion_vencida(false);
            let _ = app.emit("sincronizacion-completada", cursor);
            Ok(cursor)
        }
        Err(error) if error.status == Some(401) => {
            let _ = repo_sincronizacion
                .inner()
                .as_ref()
                .guardar_sesion_vencida(true);
            let mensaje = manejar_error(app, error);
            let _ = app.emit("sesion-vencida", &mensaje);
            Err(mensaje)
        }
        Err(error) => Err(manejar_error(app, error)),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sincronizar_grupos(app: AppHandle) -> Result<u64, String> {
    ejecutar_sincronizacion(&app).await
}

pub fn iniciar_sincronizacion_automatica(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Primer tick al abrir la aplicación y luego cada 5 minutos (300 segundos)
        let mut intervalo = tokio::time::interval(Duration::from_secs(300));

        loop {
            intervalo.tick().await;

            // Si no hay sesión activa, omite el ciclo sin emitir errores
            if obtener_token(&app).is_err() {
                continue;
            }

            let repo_sincronizacion = app.state::<Box<dyn RepositorioSincronizacion>>();
            if repo_sincronizacion
                .inner()
                .as_ref()
                .obtener_sesion_vencida()
                .unwrap_or(false)
            {
                continue;
            }

            // Los errores temporales de red (sin conexión) se ignoran y se reintentan en el siguiente ciclo
            let _ = ejecutar_sincronizacion(&app).await;
        }
    });
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
pub async fn obtener_cuenta_grupo(
    grupo_id: String,
    repo_sincronizacion: State<'_, Box<dyn RepositorioSincronizacion>>,
) -> Result<Option<CuentaGrupoSeleccionDTO>, String> {
    aplicacion::casos_uso::elegir_cuenta_grupo::obtener_cuenta_grupo(
        &grupo_id,
        repo_sincronizacion.inner().as_ref(),
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
        sesion_vencida: repo_sincronizacion
            .inner()
            .as_ref()
            .obtener_sesion_vencida()?,
    })
}