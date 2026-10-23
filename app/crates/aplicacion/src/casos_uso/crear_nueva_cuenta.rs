use crate::puertos::{RepositorioCuentas, RepositorioMonedas};
use contrato::crear_cuenta::CrearCuentaRequest;
use contrato::cuenta::CuentaDTO;
use contrato::error_validacion::ErrorValidacionDTO;
use rust_decimal::Decimal;
use std::str::FromStr;

pub async fn ejecutar(
    datos: CrearCuentaRequest,
    repo_cuentas: &dyn RepositorioCuentas,
    repo_monedas: &dyn RepositorioMonedas,
) -> Result<CuentaDTO, ErrorValidacionDTO> {
    let nombre_limpio = datos.nombre.trim();

    // 1. Validación: Nombre no vacío
    if nombre_limpio.is_empty() {
        return Err(ErrorValidacionDTO {
            campo: "nombre".to_string(),
            mensaje: "El nombre de la cuenta no puede estar vacío.".to_string(),
        });
    }

    // 2. Validación de Unicidad: Solo evaluamos los nombres existentes (case-insensitive)
    let cuentas_existentes = repo_cuentas
        .obtener_cuentas()
        .map_err(|e| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje: e,
        })?;

    let existe_nombre = cuentas_existentes
        .iter()
        .any(|c| c.nombre.trim().eq_ignore_ascii_case(nombre_limpio));

    if existe_nombre {
        return Err(ErrorValidacionDTO {
            campo: "nombre".to_string(),
            mensaje: format!("Ya existe una cuenta llamada '{}'.", nombre_limpio),
        });
    }

    // 3. Validación: Moneda dentro del catálogo base
    let monedas = repo_monedas
        .obtener_monedas()
        .map_err(|e| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje: e,
        })?;

    let moneda = monedas
        .iter()
        .find(|m| m.codigo == datos.moneda_codigo)
        .ok_or_else(|| ErrorValidacionDTO {
            campo: "moneda_codigo".to_string(),
            mensaje: format!(
                "La moneda '{}' no existe en el catálogo.",
                datos.moneda_codigo
            ),
        })?;

    // 4. Validación: Saldo inicial válido
    let saldo = Decimal::from_str(&datos.saldo_inicial).map_err(|_| ErrorValidacionDTO {
        campo: "saldo_inicial".to_string(),
        mensaje: "El saldo inicial debe ser un número decimal válido.".to_string(),
    })?;

    if saldo.is_sign_negative() {
        return Err(ErrorValidacionDTO {
            campo: "saldo_inicial".to_string(),
            mensaje: "El saldo inicial no puede ser negativo.".to_string(),
        });
    }

    // 5. Validación: Límite de decimales permitido por la moneda
    if saldo.scale() > moneda.decimales {
        return Err(ErrorValidacionDTO {
            campo: "saldo_inicial".to_string(),
            mensaje: format!(
                "El saldo para {} no puede tener más de {} decimales.",
                moneda.codigo, moneda.decimales
            ),
        });
    }

    // 6. Guardar la cuenta en SQLite y registrar el evento inmutable (CuentaCreada)
    repo_cuentas
        .guardar_cuenta(&datos)
        .map_err(|err_msg| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje: err_msg,
        })
}
