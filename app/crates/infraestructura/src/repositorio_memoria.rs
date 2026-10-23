use contrato::{CrearCuentaRequest, CuentaDTO, ErrorValidacionDTO};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Puerto del repositorio
#[allow(async_fn_in_trait)]
pub trait RepositorioEventos: Send + Sync {
    async fn guardar_cuenta(
        &self,
        request: CrearCuentaRequest,
    ) -> Result<CuentaDTO, ErrorValidacionDTO>;
    async fn listar_cuentas(&self) -> Result<Vec<CuentaDTO>, String>;
}

/// Repositorio en memoria para pruebas (sin SQLite ni red)
pub struct RepositorioMemoria {
    cuentas: Arc<Mutex<Vec<CuentaDTO>>>,
}

impl RepositorioMemoria {
    pub fn nuevo() -> Self {
        Self {
            cuentas: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl RepositorioEventos for RepositorioMemoria {
    async fn guardar_cuenta(
        &self,
        request: CrearCuentaRequest,
    ) -> Result<CuentaDTO, ErrorValidacionDTO> {
        // Simulamos una creación exitosa para que el esqueleto compile
        let nueva_cuenta = CuentaDTO {
            id: "test-id-123".to_string(),
            nombre: request.nombre,
            tipo: request.tipo,
            moneda_codigo: request.moneda_codigo,
            saldo: request.saldo_inicial,
            creada_en: "2026-10-15T12:00:00Z".to_string(),
        };

        let mut cuentas = self.cuentas.lock().await;
        cuentas.push(nueva_cuenta.clone());
        Ok(nueva_cuenta)
    }

    async fn listar_cuentas(&self) -> Result<Vec<CuentaDTO>, String> {
        let cuentas = self.cuentas.lock().await;
        Ok(cuentas.clone())
    }
}
