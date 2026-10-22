use uuid::Uuid;

use crate::errores::ErrorDominio;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoCategoria {
    Gasto,
    Ingreso,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Categoria {
    pub id: Uuid,
    pub nombre: String,
    pub tipo: TipoCategoria,
}

const GASTOS_DE_FABRICA: [&str; 7] = [
    "Alimentación",
    "Transporte",
    "Servicios",
    "Salud",
    "Educación",
    "Entretenimiento",
    "Otros gastos",
];

const INGRESOS_DE_FABRICA: [&str; 4] = ["Salario", "Ventas", "Regalos", "Otros ingresos"];

fn normalizar(nombre: &str) -> String {
    nombre.trim().to_lowercase()
}

/// Categorías incluidas de fábrica. Los ids son fijos para que sean
/// iguales en cualquier dispositivo.
pub fn catalogo_de_fabrica() -> Vec<Categoria> {
    let gastos = GASTOS_DE_FABRICA.iter().map(|n| (*n, TipoCategoria::Gasto));
    let ingresos = INGRESOS_DE_FABRICA
        .iter()
        .map(|n| (*n, TipoCategoria::Ingreso));

    gastos
        .chain(ingresos)
        .enumerate()
        .map(|(i, (nombre, tipo))| Categoria {
            id: Uuid::from_u128(i as u128 + 1),
            nombre: nombre.to_string(),
            tipo,
        })
        .collect()
}

/// Valida y construye una categoría nueva. El id llega de afuera
/// para que el dominio siga siendo puro y fácil de probar.
pub fn crear_categoria(
    existentes: &[Categoria],
    id: Uuid,
    nombre: &str,
    tipo: TipoCategoria,
) -> Result<Categoria, ErrorDominio> {
    let limpio = nombre.trim();
    if limpio.is_empty() {
        return Err(ErrorDominio::NombreCategoriaVacio);
    }

    let buscado = normalizar(limpio);
    let repetida = existentes
        .iter()
        .any(|c| c.tipo == tipo && normalizar(&c.nombre) == buscado);
    if repetida {
        return Err(ErrorDominio::CategoriaDuplicada);
    }

    Ok(Categoria {
        id,
        nombre: limpio.to_string(),
        tipo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_trae_siete_gastos_y_cuatro_ingresos() {
        let c = catalogo_de_fabrica();
        let gastos = c.iter().filter(|x| x.tipo == TipoCategoria::Gasto).count();
        let ingresos = c
            .iter()
            .filter(|x| x.tipo == TipoCategoria::Ingreso)
            .count();
        assert_eq!(gastos, 7);
        assert_eq!(ingresos, 4);
    }

    #[test]
    fn nombre_vacio_se_rechaza_en_el_campo_nombre() {
        let err =
            crear_categoria(&[], Uuid::from_u128(100), "   ", TipoCategoria::Gasto).unwrap_err();
        assert_eq!(err, ErrorDominio::NombreCategoriaVacio);
        assert_eq!(err.campo(), "nombre");
    }

    #[test]
    fn duplicada_en_el_mismo_tipo_sin_distinguir_mayusculas() {
        let existentes = catalogo_de_fabrica();
        let err = crear_categoria(
            &existentes,
            Uuid::from_u128(100),
            "  ALIMENTACIÓN ",
            TipoCategoria::Gasto,
        )
        .unwrap_err();
        assert_eq!(err, ErrorDominio::CategoriaDuplicada);
    }

    #[test]
    fn mismo_nombre_en_otro_tipo_si_se_permite() {
        let existentes = catalogo_de_fabrica();
        let nueva = crear_categoria(
            &existentes,
            Uuid::from_u128(100),
            "Salud",
            TipoCategoria::Ingreso,
        );
        assert!(nueva.is_ok());
    }

    #[test]
    fn categoria_nueva_valida_se_crea_con_nombre_limpio() {
        let nueva = crear_categoria(
            &catalogo_de_fabrica(),
            Uuid::from_u128(100),
            " Mascotas ",
            TipoCategoria::Gasto,
        )
        .unwrap();
        assert_eq!(nueva.nombre, "Mascotas");
    }
}
