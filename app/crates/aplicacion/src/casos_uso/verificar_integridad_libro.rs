use crate::puertos::{ReporteIntegridad, RepositorioEventos};

/// Verifica que el historial de eventos sea consistente y que los saldos actuales
/// coincidan exactamente con la suma de los eventos en orden determinista (RF-INT-08).
pub fn ejecutar(repo_eventos: &dyn RepositorioEventos) -> Result<ReporteIntegridad, String> {
    repo_eventos.verificar_integridad()
}
