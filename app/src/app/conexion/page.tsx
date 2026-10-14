"use client";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { usePing } from "@/features/ping/usePing";
import type { ConnectionState } from "@/types/ping";
import "../accounts.css";

const statusCopy: Record<ConnectionState["status"], string> = {
  idle: "Sin comprobar",
  loading: "Comprobando…",
  success: "Conectado",
  error: "Sin conexión",
};

export default function ConnectionPage() {
  const { state, checkConnection } = usePing();
  const response = state.status === "success" ? state.data : null;
  const checkedAt = response
    ? new Intl.DateTimeFormat("es-VE", {
        dateStyle: "short",
        timeStyle: "medium",
      }).format(new Date(response.timestamp))
    : "sin registros";

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
              <span
                className={`connection-status connection-status-${state.status}`}
                role="status"
              >
                {statusCopy[state.status]}
              </span>
            </div>
            <div className="connection-panel-body">
              <p>
                Comprueba la conexión para conocer su estado actual.
              </p>
              {state.status === "error" && (
                <p className="connection-error" role="alert">
                  {state.message}
                </p>
              )}
              {response && (
                <dl className="connection-details">
                  <div>
                    <dt>Estado del servidor</dt>
                    <dd>{response.estado}</dd>
                  </div>
                  <div>
                    <dt>Mensaje del servidor</dt>
                    <dd>{response.mensaje}</dd>
                  </div>
                  <div>
                    <dt>Versión</dt>
                    <dd>{response.version}</dd>
                  </div>
                  <div>
                    <dt>Tiempo de respuesta</dt>
                    <dd>{response.latencia_ms} ms</dd>
                  </div>
                </dl>
              )}
            </div>
            <footer className="connection-panel-footer">
              <span>Última comprobación: {checkedAt}</span>
              <button
                className="account-primary"
                type="button"
                disabled={state.status === "loading"}
                onClick={checkConnection}
              >
                {state.status === "loading"
                  ? "Comprobando…"
                  : "Verificar Conexión con el Servidor"}
              </button>
            </footer>
          </section>
        </main>
      </div>
    </div>
  );
}
