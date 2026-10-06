"use client";

import { useState } from "react";

type ConnectionStatus =
  | "disconnected"
  | "loading"
  | "connected"
  | "error";

type IconProps = {
  className?: string;
};

function ServerIcon({ className = "h-5 w-5" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      className={className}
      aria-hidden="true"
    >
      <rect x="3" y="4" width="18" height="6" rx="2" />
      <rect x="3" y="14" width="18" height="6" rx="2" />
      <path d="M7 7h.01M7 17h.01" />
    </svg>
  );
}

function ActivityIcon({ className = "h-5 w-5" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      className={className}
      aria-hidden="true"
    >
      <path d="M3 12h4l2-6 4 12 2-6h6" />
    </svg>
  );
}

function LayersIcon({ className = "h-5 w-5" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      className={className}
      aria-hidden="true"
    >
      <path d="m12 3 9 5-9 5-9-5 9-5Z" />
      <path d="m3 12 9 5 9-5" />
      <path d="m3 16 9 5 9-5" />
    </svg>
  );
}

function ClockIcon({ className = "h-5 w-5" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      className={className}
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </svg>
  );
}

function RefreshIcon({ className = "h-5 w-5" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      className={className}
      aria-hidden="true"
    >
      <path d="M20 6v5h-5" />
      <path d="M4 18v-5h5" />
      <path d="M18.5 9A7 7 0 0 0 6.2 6.2L4 8" />
      <path d="M5.5 15A7 7 0 0 0 17.8 17.8L20 16" />
    </svg>
  );
}

export default function Home() {
  const [status, setStatus] =
    useState<ConnectionStatus>("disconnected");

  const [lastCheck, setLastCheck] = useState<string | null>(null);

  const handleCheckConnection = () => {
    setStatus("loading");

    setTimeout(() => {
      setStatus("connected");

      setLastCheck(
        new Intl.DateTimeFormat("es-VE", {
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
        }).format(new Date()),
      );
    }, 1200);
  };

  const statusConfig = {
    disconnected: {
      label: "Desconectado",
      description: "Aún no se ha verificado la conexión.",
      dot: "bg-slate-400",
      badge:
        "border-slate-700 bg-slate-800/70 text-slate-300",
      glow: "bg-slate-500/10",
    },

    loading: {
      label: "Verificando",
      description: "Comprobando comunicación con el servidor...",
      dot: "bg-amber-400 animate-pulse",
      badge:
        "border-amber-400/20 bg-amber-400/10 text-amber-300",
      glow: "bg-amber-400/10",
    },

    connected: {
      label: "Conectado",
      description: "El servidor está operativo y respondiendo.",
      dot: "bg-emerald-400",
      badge:
        "border-emerald-400/20 bg-emerald-400/10 text-emerald-300",
      glow: "bg-emerald-400/10",
    },

    error: {
      label: "Error",
      description: "No fue posible establecer conexión.",
      dot: "bg-red-400",
      badge:
        "border-red-400/20 bg-red-400/10 text-red-300",
      glow: "bg-red-400/10",
    },
  };

  const currentStatus = statusConfig[status];

  return (
    <main className="relative min-h-screen overflow-hidden bg-[#070b14] text-white">
      {/* Decorative background */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute -left-40 -top-40 h-[500px] w-[500px] rounded-full bg-blue-600/10 blur-[120px]" />

        <div className="absolute -right-40 top-1/3 h-[500px] w-[500px] rounded-full bg-cyan-400/10 blur-[130px]" />

        <div className="absolute bottom-[-250px] left-1/3 h-[500px] w-[500px] rounded-full bg-indigo-500/10 blur-[140px]" />

        <div
          className="absolute inset-0 opacity-[0.035]"
          style={{
            backgroundImage:
              "linear-gradient(rgba(255,255,255,.8) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.8) 1px, transparent 1px)",
            backgroundSize: "48px 48px",
          }}
        />
      </div>

      <div className="relative mx-auto flex min-h-screen w-full max-w-7xl flex-col px-5 py-6 sm:px-8 lg:px-12">
        {/* Header */}
        <header className="flex items-center justify-between border-b border-white/[0.08] pb-5">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-blue-400/20 bg-blue-500/10 shadow-[0_0_30px_rgba(59,130,246,0.12)]">
              <ActivityIcon className="h-5 w-5 text-blue-300" />
            </div>

            <div>
              <p className="text-sm font-semibold tracking-wide text-white">
                Cuentas Claras
              </p>

              <p className="text-xs text-slate-500">
                Connection Monitor
              </p>
            </div>
          </div>

          <div className="rounded-full border border-white/[0.08] bg-white/[0.04] px-3 py-1.5 text-xs text-slate-400">
            v0.1.0
          </div>
        </header>

        {/* Main content */}
        <section className="flex flex-1 items-center py-12 sm:py-16">
          <div className="grid w-full gap-10 lg:grid-cols-[1.05fr_0.95fr] lg:items-center">
            {/* Hero */}
            <div className="max-w-2xl">
              <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-blue-400/20 bg-blue-400/[0.06] px-3 py-1.5 text-xs font-medium text-blue-300">
                <span className="h-1.5 w-1.5 rounded-full bg-blue-400" />
                Estado del sistema
              </div>

              <h1 className="max-w-xl text-4xl font-semibold leading-tight tracking-[-0.03em] text-white sm:text-5xl lg:text-6xl">
                Todo claro.
                <span className="block bg-gradient-to-r from-blue-300 via-cyan-300 to-indigo-300 bg-clip-text text-transparent">
                  Todo conectado.
                </span>
              </h1>

              <p className="mt-6 max-w-xl text-base leading-7 text-slate-400 sm:text-lg">
                Comprueba en segundos que la aplicación puede comunicarse
                correctamente con el servidor de Cuentas Claras.
              </p>

              <div className="mt-8 flex flex-wrap gap-3 text-sm text-slate-400">
                <div className="flex items-center gap-2 rounded-full border border-white/[0.07] bg-white/[0.03] px-3 py-2">
                  <span className="h-1.5 w-1.5 rounded-full bg-blue-400" />
                  Next.js
                </div>

                <div className="flex items-center gap-2 rounded-full border border-white/[0.07] bg-white/[0.03] px-3 py-2">
                  <span className="h-1.5 w-1.5 rounded-full bg-violet-400" />
                  Tauri
                </div>

                <div className="flex items-center gap-2 rounded-full border border-white/[0.07] bg-white/[0.03] px-3 py-2">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
                  FastAPI
                </div>
              </div>
            </div>

            {/* Connection panel */}
            <div className="relative">
              <div
                className={`absolute -inset-10 rounded-full blur-[80px] transition-colors duration-500 ${currentStatus.glow}`}
              />

              <div className="relative overflow-hidden rounded-[28px] border border-white/[0.09] bg-[#0c111d]/90 shadow-2xl shadow-black/30 backdrop-blur-xl">
                <div className="border-b border-white/[0.07] px-6 py-5 sm:px-7">
                  <div className="flex items-center justify-between gap-4">
                    <div>
                      <p className="text-xs font-medium uppercase tracking-[0.18em] text-slate-500">
                        Servidor
                      </p>

                      <h2 className="mt-1 text-lg font-semibold text-white">
                        Estado de conexión
                      </h2>
                    </div>

                    <ServerIcon className="h-6 w-6 text-slate-500" />
                  </div>
                </div>

                <div className="p-6 sm:p-7">
                  {/* Status */}
                  <div className="rounded-2xl border border-white/[0.07] bg-white/[0.025] p-5">
                    <div className="flex items-start justify-between gap-5">
                      <div>
                        <p className="text-sm text-slate-500">
                          Estado actual
                        </p>

                        <div className="mt-3 flex items-center gap-3">
                          <span
                            className={`h-2.5 w-2.5 rounded-full ${currentStatus.dot}`}
                          />

                          <span className="text-xl font-semibold text-white">
                            {currentStatus.label}
                          </span>
                        </div>

                        <p className="mt-2 text-sm leading-6 text-slate-400">
                          {currentStatus.description}
                        </p>
                      </div>

                      <span
                        className={`rounded-full border px-3 py-1 text-xs font-medium ${currentStatus.badge}`}
                      >
                        {status === "connected"
                          ? "ONLINE"
                          : status === "loading"
                            ? "CHECKING"
                            : status === "error"
                              ? "ERROR"
                              : "OFFLINE"}
                      </span>
                    </div>
                  </div>

                  {/* Button */}
                  <button
                    type="button"
                    onClick={handleCheckConnection}
                    disabled={status === "loading"}
                    className="group mt-5 flex w-full items-center justify-center gap-2.5 rounded-2xl bg-gradient-to-r from-blue-600 to-blue-500 px-5 py-3.5 text-sm font-semibold text-white shadow-lg shadow-blue-950/30 transition duration-200 hover:-translate-y-0.5 hover:from-blue-500 hover:to-cyan-500 hover:shadow-blue-900/40 disabled:cursor-not-allowed disabled:translate-y-0 disabled:opacity-60"
                  >
                    <RefreshIcon
                      className={`h-4 w-4 ${
                        status === "loading" ? "animate-spin" : ""
                      }`}
                    />

                    {status === "loading"
                      ? "Verificando conexión..."
                      : "Verificar conexión con el servidor"}
                  </button>

                  {/* Metrics */}
                  <div className="mt-6 grid gap-3 sm:grid-cols-3">
                    <MetricCard
                      icon={
                        <ActivityIcon className="h-4 w-4 text-blue-300" />
                      }
                      label="Mensaje"
                      value={
                        status === "connected"
                          ? "Servidor activo"
                          : "Sin verificar"
                      }
                    />

                    <MetricCard
                      icon={
                        <LayersIcon className="h-4 w-4 text-violet-300" />
                      }
                      label="Versión"
                      value={status === "connected" ? "0.1.0" : "--"}
                    />

                    <MetricCard
                      icon={
                        <ClockIcon className="h-4 w-4 text-cyan-300" />
                      }
                      label="Latencia"
                      value={status === "connected" ? "42 ms" : "--"}
                    />
                  </div>

                  <div className="mt-5 flex items-center justify-between border-t border-white/[0.07] pt-5 text-xs text-slate-500">
                    <span>Última verificación</span>

                    <span className="font-medium text-slate-400">
                      {lastCheck ?? "Todavía no realizada"}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <footer className="flex flex-col gap-2 border-t border-white/[0.07] py-5 text-xs text-slate-600 sm:flex-row sm:items-center sm:justify-between">
          <span>Cuentas Claras · Sistema de conexión</span>

          <span>Entrega 1 · Full Stack Ping</span>
        </footer>
      </div>
    </main>
  );
}

type MetricCardProps = {
  icon: React.ReactNode;
  label: string;
  value: string;
};

function MetricCard({ icon, label, value }: MetricCardProps) {
  return (
    <div className="rounded-2xl border border-white/[0.07] bg-white/[0.025] p-4 transition hover:border-white/[0.12] hover:bg-white/[0.04]">
      <div className="flex items-center gap-2 text-slate-500">
        {icon}
        <span className="text-xs">{label}</span>
      </div>

      <p className="mt-3 truncate text-sm font-semibold text-slate-200">
        {value}
      </p>
    </div>
  );
}