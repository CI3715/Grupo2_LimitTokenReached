"use client";

import { ConnectionButton } from "@/components/ConnectionButton";
import { InfoCard } from "@/components/InfoCard";
import { LatencyBadge } from "@/components/LatencyBadge";
import { StatusIndicator } from "@/components/StatusIndicator";
import { usePing } from "@/features/ping/usePing";
import type { ConnectionState, PingResponse } from "@/types/ping";

function getStatus(state: ConnectionState): "idle" | "loading" | "success" | "error" {
  return state.status;
}

function getResponse(state: ConnectionState): PingResponse | null {
  return state.status === "success" ? state.data : null;
}

function formatTimestamp(timestamp: string | undefined): string {
  if (!timestamp) {
    return "No realizada";
  }

  return new Intl.DateTimeFormat("es-VE", {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(timestamp));
}

export default function Home() {
  const { state, checkConnection } = usePing();
  const response = getResponse(state);
  const status = getStatus(state);

  return (
    <main
      data-theme={theme}
      className="app-shell relative min-h-screen overflow-hidden"
    >
      <NetworkBackground
        status={status}
        theme={theme}
      />

      {/* ===================================================
          AMBIENT BACKGROUND
      ==================================================== */}

      <div
        className="pointer-events-none fixed inset-0 z-0 overflow-hidden"
        aria-hidden="true"
      >
        <div className="ambient-blob ambient-blob-one" />

        <div className="ambient-blob ambient-blob-two" />

        <div className="ambient-blob ambient-blob-three" />

        <div className="ambient-ring ambient-ring-one" />

        <div className="ambient-ring ambient-ring-two ambient-ring-delay" />

        <div className="ambient-ring ambient-ring-three ambient-ring-slow" />
      </div>

      <div
        className="center-light pointer-events-none fixed inset-0 z-0"
        aria-hidden="true"
      />

      <div
        className="bottom-fade pointer-events-none fixed inset-0 z-0"
        aria-hidden="true"
      />

      <div
        className="noise-layer pointer-events-none fixed inset-0 z-0"
        aria-hidden="true"
      />

      {/* ===================================================
          APPLICATION
      ==================================================== */}

      <div className="relative z-10 mx-auto flex min-h-screen w-full max-w-[1500px] flex-col px-5 sm:px-8 lg:px-12">
        {/* HEADER */}

        <header className="header-border flex h-20 items-center justify-between">
          <div className="flex items-center gap-3">
            <IOSModuleIcon className="h-11 w-11" />

            <div>
              <p className="primary-text text-sm font-semibold tracking-[-0.01em]">
                Cuentas Claras
              </p>

              <p className="secondary-soft mt-0.5 text-[10px] uppercase tracking-[0.18em]">
                Digital System
              </p>
            </div>
          </div>
          <StatusIndicator status={status} />
        </header>

        <section className="grid flex-1 items-center gap-10 py-12 lg:grid-cols-[1fr_1.2fr]">
          <div>
            <p className="eyebrow-text text-xs font-semibold uppercase tracking-[0.25em]">
              Verificación de infraestructura
            </p>
            <h1 className="hero-title mt-5 text-5xl font-medium tracking-[-0.06em] sm:text-7xl">
              Claridad
              <span className="hero-gradient block">conectada.</span>
            </h1>
            <p className="secondary-text mt-6 max-w-lg text-sm leading-7">
              Comprueba la comunicación entre la interfaz Next.js, el puente
              local de Tauri y el servidor FastAPI.
            </p>

            <div className="mt-8">
              <ConnectionButton
                isLoading={status === "loading"}
                isConnected={status === "success"}
                onCheckConnection={checkConnection}
              />
            </div>

            <ThemeToggle
              theme={theme}
              onToggle={
                toggleTheme
              }
            />

            <span className="secondary-soft hidden font-mono text-[10px] tracking-[0.12em] sm:inline">
              v0.1.0
            </span>
          </div>
        </header>

        <section className="grid flex-1 items-center gap-10 py-12 lg:grid-cols-[1fr_1.2fr]">
          <div>
            <p className="eyebrow-text text-xs font-semibold uppercase tracking-[0.25em]">
              Verificación de infraestructura
            </p>
            <h1 className="hero-title mt-5 text-5xl font-medium tracking-[-0.06em] sm:text-7xl">
              Claridad
              <span className="hero-gradient block">conectada.</span>
            </h1>
            <p className="secondary-text mt-6 max-w-lg text-sm leading-7">
              Comprueba la comunicación entre la interfaz Next.js, el puente
              local de Tauri y el servidor FastAPI.
            </p>

            <div className="mt-8">
              <ConnectionButton
                isLoading={status === "loading"}
                isConnected={status === "success"}
                onCheckConnection={checkConnection}
              />
            </div>

            {state.status === "error" && (
              <p className="mt-4 max-w-sm text-sm text-red-400" role="alert">
                {state.message}
              </p>
            )}
          </div>

          <section className="system-card rounded-[32px] p-6 sm:p-9" aria-live="polite">
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div>
                <p className="secondary-soft text-[10px] uppercase tracking-[0.24em]">
                  Core connection
                </p>
                <h2 className="primary-text mt-2 text-2xl font-semibold">
                  {status === "loading"
                    ? "Conectando sistemas"
                    : status === "success"
                      ? "Todo conectado"
                      : status === "error"
                        ? "Conexión interrumpida"
                        : "Listo para verificar"}
                </h2>
              </div>
              <StatusIndicator status={status} compact />
            </div>

            <div className="mt-8 grid gap-3 sm:grid-cols-2">
              <InfoCard
                label="Estado"
                value={response?.estado ?? (status === "loading" ? "Consultando..." : "--")}
              />
              <InfoCard
                label="Mensaje del servidor"
                value={response?.mensaje ?? "Sin respuesta"}
              />
              <InfoCard label="Versión" value={response?.version ?? "--"} />
              <InfoCard
                label="Última sincronización"
                value={formatTimestamp(response?.timestamp)}
              />
            </div>

                {/* LAST SYNC */}

                <div className="panel-divider secondary-soft relative z-10 mt-5 flex flex-col gap-3 border-t pt-5 text-[10px] uppercase tracking-[0.14em] sm:flex-row sm:items-center sm:justify-between">
                  <span>
                    Última
                    sincronización
                  </span>

                  <span className="font-mono">
                    {lastCheck ??
                      "No realizada"}
                  </span>
                </div>
              </div>
            </div>
          </section>
        </section>

        <footer className="footer-border border-t py-5 text-[10px] uppercase tracking-[0.15em]">
          <span className="secondary-soft">Next.js · Tauri · FastAPI</span>
        </footer>
      </div>
    </main>
  );
}
