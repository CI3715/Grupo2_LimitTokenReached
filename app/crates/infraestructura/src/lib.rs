use aplicacion::puertos::Reloj; // Importa el Trait definido en aplicacion
use chrono::Utc;
// Módulo para el repositorio en memoria (Pruebas QA)
pub mod repositorio_memoria;
//

pub mod autenticacion_http;
pub mod bd;
pub mod repositorio_categorias_sqlite;
pub mod repositorio_cuentas_sqlite;
pub mod repositorio_monedas_sqlite;
pub mod repositorio_movimientos_sqlite;
pub mod repositorio_patrimonio_sqlite;
pub mod repositorio_preferencias_sqlite;
pub mod repositorio_tasas_sqlite;
pub mod social_http;

pub use autenticacion_http::{
    ClienteAutenticacionHttp, ErrorApi, InicioSesionHttp, RegistroUsuarioHttp, SesionHttp,
    UsuarioHttp,
};
pub use repositorio_categorias_sqlite::RepositorioCategoriasSqlite;
pub use repositorio_cuentas_sqlite::RepositorioCuentasSqlite;
pub use repositorio_monedas_sqlite::RepositorioMonedasSqlite;
pub use repositorio_movimientos_sqlite::RepositorioMovimientosSqlite;
pub use repositorio_patrimonio_sqlite::RepositorioPatrimonioSqlite;
pub use repositorio_preferencias_sqlite::RepositorioPreferenciasSqlite;
pub use repositorio_tasas_sqlite::RepositorioTasasSqlite;
pub use social_http::{ClienteSocialHttp, PingRespuesta, SocialError};

// Adaptador Real del Reloj del Sistema
pub struct RelojSistema;

impl Reloj for RelojSistema {
    fn ahora(&self) -> String {
        Utc::now().to_rfc3339() // Devuelve la fecha y hora real UTC en formato ISO 8601
    }
}
