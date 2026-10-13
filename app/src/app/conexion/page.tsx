import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import "../accounts.css";

export default function ConnectionPage() {
  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="connection" />
        <main className="accounts-workspace">
          <header className="accounts-header">
            <div>
              <p className="breadcrumb">
                Sistema <span>/</span> Conexión
              </p>
              <h1>Conexión</h1>
              <p>Esta sección estará disponible próximamente.</p>
            </div>
          </header>
          <section
            className="connection-panel"
            aria-labelledby="connection-title"
          >
            <div className="connection-panel-heading">
              <div className="connection-server-icon" aria-hidden="true">
                ⌁
              </div>
              <div>
                <h2 id="connection-title">Estado de conexión</h2>
                <p>Vista previa</p>
              </div>
              <span className="connection-status">Próximamente</span>
            </div>
            <div className="connection-panel-body">
              <p>
                Por ahora puedes explorar Mis cuentas y probar su formulario con
                datos de muestra.
              </p>
            </div>
            <footer className="connection-panel-footer">
              <span>Esta vista no realiza comprobaciones.</span>
              <button className="account-primary" disabled>
                Comprobar conexión
              </button>
            </footer>
          </section>
        </main>
      </div>
    </div>
  );
}
