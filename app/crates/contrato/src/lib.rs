pub mod crear_cuenta;
pub mod cuenta;
pub mod error_validacion;
pub mod moneda;
pub mod movimiento;
pub mod categoria;

// Re-exportaciones para uso cómodo desde otros crates (aplicacion, infraestructura, frontend)
pub use crear_cuenta::CrearCuentaRequest;
pub use cuenta::{CuentaDTO, TipoCuentaDTO};
pub use error_validacion::ErrorValidacionDTO;
pub use moneda::MonedaDTO;
pub use movimiento::{
    CorregirMovimientoRequest, MovimientoDTO, RegistrarMovimientoRequest,
    TipoMovimientoDTO,
};

pub use categoria::{CategoriaDTO, CrearCategoriaRequest, TipoCategoriaDTO};

// Módulos de pruebas
#[cfg(test)]
mod crear_cuenta_tests;

#[cfg(test)]
mod movimiento_tests;
