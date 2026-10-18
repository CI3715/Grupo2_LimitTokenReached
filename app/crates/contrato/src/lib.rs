pub mod cuenta;
pub mod crear_cuenta;
pub mod error_validacion;
pub mod moneda;
pub mod movimiento;

// Re-exportaciones para uso cómodo desde otros crates (aplicacion, infraestructura, frontend)
pub use cuenta::{CuentaDTO, TipoCuentaDTO};
pub use crear_cuenta::CrearCuentaRequest;
pub use error_validacion::ErrorValidacionDTO;
pub use moneda::MonedaDTO;
pub use movimiento::{
    CategoriaDTO, CorregirMovimientoRequest, MovimientoDTO,
    RegistrarMovimientoRequest, TipoMovimientoDTO,
};

// Módulos de pruebas
#[cfg(test)]
mod crear_cuenta_tests;

#[cfg(test)]
mod movimiento_tests;