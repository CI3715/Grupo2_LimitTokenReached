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
              <p>Consulta el estado de conexión de Cuentas Claras.</p>
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
                <p>Conexión de la aplicación</p>
              </div>
              <span className="connection-status">Sin comprobar</span>
            </div>
            <div className="connection-panel-body">
              <p>
                Comprueba la conexión para conocer su estado actual.
              </p>
            </div>
            <footer className="connection-panel-footer">
              <span>Última comprobación: sin registros</span>
              <button className="account-primary" type="button">
                Comprobar conexión
              </button>
            </footer>
          </section>
        </main>
      </div>
    </div>
  );
}
