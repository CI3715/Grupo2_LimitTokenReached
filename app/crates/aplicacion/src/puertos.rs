// Puerto para abstenerse de llamadas directas al reloj del SO
// Permite inyectar horas ficticias en tests automatizados.

pub trait Reloj: Send + {
    fn ahora(&self) -> String;

}


// Puerto para abstracción de solicitudes HTTP salientes hacia el Servidor FastAPI
pub trait ClienteSocial: Send + Sync {
    fn hacer_ping(&self) -> impl std::future::Future<Output = Result<String, String>> + Send;
}



// Adapradores Fake

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RelojPrueba;

#[cfg(test)]
impl Reloj for RelojPrueba {
    fn ahora(&self) -> String {
        "2026-10-06T12:00:00Z".to_string() // Fecha fija e inmutable para tests
    }
}