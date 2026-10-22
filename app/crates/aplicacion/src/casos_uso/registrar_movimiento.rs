use crate::puertos::{RepositorioMovimientos, RepositorioCuentas, RepositorioMonedas, Reloj};
use dominio::movimiento::{registrar_movimiento as registrar_movimiento_dominio, DatosMovimiento, TipoMovimiento, MonedaCuenta};
use dominio::errores::ErrorDominio;
use contrato::movimiento::{RegistrarMovimientoRequest, TipoMovimientoDTO, MovimientoDTO};
use contrato::error_validacion::ErrorValidacionDTO;
use uuid::Uuid;
use std::str::FromStr;
use rust_decimal::Decimal;
use chrono::NaiveDate;

/// Ejecuta el registro de un movimiento orquestando las validaciones puras del dominio
pub async fn ejecutar(
    datos: RegistrarMovimientoRequest,
    repo_movimientos: &dyn RepositorioMovimientos,
    repo_cuentas: &dyn RepositorioCuentas,
    repo_monedas: &dyn RepositorioMonedas,
    reloj: &dyn Reloj,
) -> Result<MovimientoDTO, ErrorValidacionDTO> {

    // 1. Transformar los datos primitivos del DTO a los tipos requeridos
    let cuenta_id = Uuid::from_str(&datos.cuenta_id)
        .map_err(|_| ErrorValidacionDTO {
            campo: "cuenta_id".to_string(),
            mensaje: "ID de cuenta inválido".to_string(),
        })?;
        
    let monto = Decimal::from_str(&datos.monto)
        .map_err(|_| ErrorValidacionDTO {
            campo: "monto".to_string(),
            mensaje: "El monto debe ser un número decimal válido".to_string(),
        })?;
        
    let fecha = NaiveDate::from_str(&datos.fecha)
        .map_err(|_| ErrorValidacionDTO {
            campo: "fecha".to_string(),
            mensaje: "Formato de fecha inválido. Use AAAA-MM-DD".to_string(),
        })?;
        
    let categoria_id = match &datos.categoria_id {
        Some(id_str) => Some(Uuid::from_str(id_str).map_err(|_| ErrorValidacionDTO {
            campo: "categoria_id".to_string(),
            mensaje: "ID de categoría inválido".to_string(),
        })?),
        None => None,
    };

    // Mapeo directo del enum TipoMovimientoDTO (Contrato) al del Dominio
   let tipo_movimiento = match datos.tipo {
        TipoMovimientoDTO::Ingreso => TipoMovimiento::Ingreso,
        TipoMovimientoDTO::Gasto => TipoMovimiento::Gasto,
        TipoMovimientoDTO::Ajuste => {
            return Err(ErrorValidacionDTO {
                campo: "tipo".to_string(),
                mensaje: "Los ajustes deben registrarse mediante el caso de uso de corrección."
                    .to_string(),
            });
        }
    };
    
    // 2. Obtener la fecha actual desde el puerto del Reloj
    let hoy_str = reloj.ahora(); 
    let hoy = hoy_str[..10].parse::<NaiveDate>().map_err(|_| ErrorValidacionDTO {
        campo: "fecha".to_string(),
        mensaje: "Error al leer el reloj del sistema".to_string(),
    })?;

    // 3. Obtener los datos accesorios (Moneda y Categorías) a través de los puertos
  let cuentas = repo_cuentas.obtener_cuentas().map_err(|mensaje| {
        ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        }
    })?;
    let cuenta = cuentas
        .into_iter()
        .find(|c| c.id == datos.cuenta_id)
        .ok_or_else(|| ErrorValidacionDTO {
            campo: "cuenta_id".to_string(),
            mensaje: format!("No se encontró la cuenta con ID: {}", datos.cuenta_id),
        })?;

    let catalogo_monedas = repo_monedas.obtener_monedas().map_err(|mensaje| {
        ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        }
    
    })?;
    let moneda_dto = catalogo_monedas
        .iter()
        .find(|m| m.codigo == cuenta.moneda_codigo)
        .ok_or_else(|| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje: "La moneda de la cuenta no existe en el catálogo".to_string(),
        })?;

    let moneda_cuenta = MonedaCuenta {
        codigo: moneda_dto.codigo.clone(),
        decimales: moneda_dto.decimales,
    };

    let categorias = repo_movimientos.obtener_categorias().map_err(|mensaje| {
        ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        }
    })?;

    // 4. Construir la estructura de entrada para el dominio
    let datos_dominio = DatosMovimiento {
        cuenta_id,
        tipo: tipo_movimiento,
        monto,
        fecha,
        categoria_id,
        nota: datos.nota.clone(),
    };

    let nuevo_id = Uuid::new_v4();

    // 5. INVOCAR LA LÓGICA PURA DE DOMINIO Y MAPEAR ERRORES A ERRORVALIDACIONDTO
    let movimiento_validado = registrar_movimiento_dominio(
        nuevo_id,
        datos_dominio,
        &moneda_cuenta,
        hoy,
        &categorias,
    ).map_err(|err| match err {
        ErrorDominio::MontoNoPositivo => ErrorValidacionDTO {
            campo: "monto".to_string(),
            mensaje: "El monto debe ser mayor a cero.".to_string(),
        },
        ErrorDominio::DecimalesExcedidos { moneda, maximo } => ErrorValidacionDTO {
            campo: "monto".to_string(),
            mensaje: format!("La moneda {} admite un máximo de {} decimales.", moneda, maximo),
        },
        ErrorDominio::CategoriaVacia => ErrorValidacionDTO {
            campo: "categoria_id".to_string(),
            mensaje: "Debe seleccionar una categoría.".to_string(),
        },
        ErrorDominio::CategoriaInexistente => ErrorValidacionDTO {
            campo: "categoria_id".to_string(),
            mensaje: "La categoría seleccionada no existe.".to_string(),
        },
        ErrorDominio::CategoriaTipoIncorrecto => ErrorValidacionDTO {
            campo: "categoria_id".to_string(),
            mensaje: "El tipo de categoría no coincide con el tipo de movimiento.".to_string(),
        },
        ErrorDominio::FechaFutura => ErrorValidacionDTO {
            campo: "fecha".to_string(),
            mensaje: "La fecha no puede ser futura.".to_string(),
        },
        _ => ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje: format!("Error de validación: {:?}", err),
        },
    })?;

    // 6. Delegar la persistencia física al adaptador de infraestructura
    repo_movimientos
        .guardar_movimiento(&movimiento_validado)
        .map_err(|mensaje| ErrorValidacionDTO {
            campo: "general".to_string(),
            mensaje,
        })
}