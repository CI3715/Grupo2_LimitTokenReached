use aplicacion::puertos::RepositorioPreferencias;
use infraestructura::{RepositorioPreferenciasSqlite, bd};
use uuid::Uuid;

#[test]
fn conserva_la_moneda_base_al_reabrir_la_base_de_datos() {
    let ruta = std::env::temp_dir().join(format!(
        "cuentas-claras-preferencias-{}.db",
        Uuid::new_v4()
    ));

    let repo = RepositorioPreferenciasSqlite::new(bd::abrir(&ruta).unwrap());
    assert_eq!(repo.obtener_moneda_base().unwrap(), "USD");
    repo.guardar_moneda_base("GBP").unwrap();
    drop(repo);

    let repo = RepositorioPreferenciasSqlite::new(bd::abrir(&ruta).unwrap());
    assert_eq!(repo.obtener_moneda_base().unwrap(), "GBP");
    drop(repo);

    std::fs::remove_file(&ruta).unwrap();
    for extension in ["-wal", "-shm"] {
        let mut sidecar = ruta.as_os_str().to_os_string();
        sidecar.push(extension);
        let sidecar = std::path::PathBuf::from(sidecar);
        if sidecar.exists() {
            std::fs::remove_file(sidecar).unwrap();
        }
    }
}
