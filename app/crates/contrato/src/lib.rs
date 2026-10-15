pub mod cuenta;
pub mod crear_cuenta;
pub mod error_validacion;
pub mod moneda;

// Re-exportaciones para uso cómodo
pub use cuenta::{CuentaDTO, TipoCuentaDTO};
pub use crear_cuenta::CrearCuentaRequest;
pub use error_validacion::ErrorValidacionDTO;
pub use moneda::MonedaDTO;

#[cfg(test)]
mod crear_cuenta_tests;