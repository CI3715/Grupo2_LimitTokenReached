"use client";

import { useState } from "react";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { pingServer } from "@/services/pingService";
import type { ConnectionState } from "@/types/ping";
import "../accounts.css";

export default function ConnectionPage() {
  const [state, setState] = useState<ConnectionState>({ status: "idle" });
  const [lastCheck, setLastCheck] = useState<string>();
  const loading = state.status === "loading";
  const labels = {
    idle: "Sin comprobar",
    loading: "Comprobando…",
    success: "Conectado",
    error: "No disponible",
  };

  async function checkConnection() {
    if (loading) return;
    setState({ status: "loading" });
    try {
      const data = await pingServer();
      setState({ status: "success", data });
    } catch (cause) {
      setState({
        status: "error",
        message:
          cause instanceof Error
            ? cause.message
            : "No pudimos conectar con el servidor. Comprueba que esté encendido e inténtalo de nuevo.",
      });
    } finally {
      setLastCheck(
        new Intl.DateTimeFormat("es-VE", {
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
          timeZone: "America/Caracas",
        }).format(new Date()),
      );
    }
  }

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
              <p>Comprueba la disponibilidad del servidor de Cuentas Claras.</p>
            </div>
          </header>
          <section
            className="connection-panel"
            aria-labelledby="server-title"
            aria-busy={loading}
          >
            <div className="connection-panel-heading">
              <div className="connection-server-icon" aria-hidden="true">
                <svg
                  width="25"
                  height="25"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                >
                  <rect x="4" y="3" width="16" height="7" rx="2" />
                  <rect x="4" y="14" width="16" height="7" rx="2" />
                  <path d="M8 6.5h.01M8 17.5h.01M12 6.5h4M12 17.5h4" />
                </svg>
              </div>
              <div>
                <h2 id="server-title">Servidor</h2>
                <p>Estado de la conexión</p>
              </div>
              <span
                className={`connection-status connection-status-${state.status}`}
                role="status"
              >
                <i aria-hidden="true" />
                {labels[state.status]}
              </span>
            </div>
            <div className="connection-panel-body">
              {state.status === "idle" && (
                <p>
                  Todavía no has comprobado la conexión. Puedes hacerlo cuando
                  lo necesites.
                </p>
              )}
              {loading && <p>Esperando la respuesta del servidor…</p>}
              {state.status === "error" && (
                <p className="connection-error" role="alert">
                  {state.message}
                </p>
              )}
              {state.status === "success" && (
                <>
                  <p>{state.data.mensaje}</p>
                  <dl className="connection-details">
                    <div>
                      <dt>Versión del servidor</dt>
                      <dd>{state.data.version}</dd>
                    </div>
                    <div>
                      <dt>Tiempo de respuesta</dt>
                      <dd>
                        {new Intl.NumberFormat("es-VE", {
                          maximumFractionDigits: 1,
                        }).format(state.data.latencia_ms)}{" "}
                        ms
                      </dd>
                    </div>
                  </dl>
                </>
              )}
            </div>
            <footer className="connection-panel-footer">
              <span>
                {lastCheck
                  ? `Última comprobación: ${lastCheck}`
                  : "Sin comprobaciones recientes"}
              </span>
              <button
                className="account-primary"
                onClick={() => void checkConnection()}
                disabled={loading}
              >
                {loading
                  ? "Comprobando…"
                  : "Verificar conexión con el servidor"}
              </button>
            </footer>
          </section>
          <aside className="connection-local-info">
            <span aria-hidden="true">✓</span>
            <div>
              <h2>Mis cuentas funciona sin conexión</h2>
              <p>
                El servidor no participa en el registro de tus cuentas. Tus
                saldos se mantienen en este dispositivo.
              </p>
            </div>
          </aside>
        </main>
      </div>
    </div>
  );
}
