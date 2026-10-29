pub mod categoria;
pub mod crear_cuenta;
pub mod cuenta;
pub mod error_validacion;
pub mod moneda;
pub mod movimiento;
pub mod patrimonio;
pub mod tasa;

// Re-exportaciones para uso cómodo desde otros crates
pub use categoria::{CategoriaDTO, CrearCategoriaRequest, TipoCategoriaDTO};
pub use crear_cuenta::CrearCuentaRequest;
pub use cuenta::{CuentaDTO, TipoCuentaDTO};
pub use error_validacion::ErrorValidacionDTO;
pub use moneda::{CambiarMonedaBaseRequest, CrearMonedaRequest, MonedaDTO};
pub use movimiento::{
    CorregirMovimientoRequest, MovimientoDTO, RegistrarMovimientoRequest, TipoMovimientoDTO,
};
pub use patrimonio::{
    ConsultarPatrimonioRequest, ConsultarPatrimonioResponse, SaldoCuentaDTO, SubtotalMonedaDTO,
    TasaFaltanteDTO, TasaUsadaDTO,
};
pub use tasa::{EliminarTasaRequest, RegistrarTasaRequest, TasaDTO};

// Módulos de pruebas
#[cfg(test)]
mod crear_cuenta_tests;

#[cfg(test)]
mod movimiento_tests;

#[cfg(test)]
mod tasa_tests;

#[cfg(test)]
mod patrimonio_tests;
