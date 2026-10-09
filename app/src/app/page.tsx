"use client";

import type {ConnectionStatus, Theme, ConnectionData, Particle} from "./components/tipes";
import NetworkBackground from "./components/NetworkBackground";
import StatusOrb from "./components/StatusOrb";
import SystemFlow from "./components/SystemFlow";
import DataItem from "./components/cards/DataItem";
import ThemeToggle from "./components/ThemeToggle";
import IOSModuleIcon from "./components/IOSModuleIcon";
import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { timeStamp } from "console";


/* =========================================================
   IOS / VISIONOS STYLE ICON
========================================================= */



/* =========================================================
   HOME
========================================================= */

export default function Home() {
  const [theme, setTheme] =
    useState<Theme>("dark");

  const [status, setStatus] =
    useState<ConnectionStatus>(
      "disconnected",
    );

  const [data, setData] =
    useState<ConnectionData | null>(
      null,
    );

  const [
    lastCheck,
    setLastCheck,
  ] =
    useState<string | null>(
      null,
    );

  /*
    Cargamos el tema guardado.

    Si el usuario todavía no eligió uno,
    usamos el tema del sistema operativo.
  */

  useEffect(() => {
  const frame = window.requestAnimationFrame(() => {
    const savedTheme =
      window.localStorage.getItem(
        "cuentas-claras-theme",
      );

    if (
      savedTheme === "dark" ||
      savedTheme === "light"
    ) {
      setTheme(savedTheme);
      return;
    }

    const prefersLight =
      window.matchMedia(
        "(prefers-color-scheme: light)",
      ).matches;

    setTheme(
      prefersLight
        ? "light"
        : "dark",
    );
  });

  return () => {
    window.cancelAnimationFrame(frame);
  };
}, []);

  /*
    Sincronizamos también el fondo real
    del documento para evitar bordes o
    flashes oscuros al usar light mode.
  */

  useEffect(() => {
    const background =
      theme === "light"
        ? "#f4f7fc"
        : "#03060d";

    document.documentElement.style.backgroundColor =
      background;

    document.body.style.backgroundColor =
      background;

    document.documentElement.style.colorScheme =
      theme;
  }, [theme]);

  const toggleTheme = () => {
    const nextTheme: Theme =
      theme === "dark"
        ? "light"
        : "dark";

    setTheme(nextTheme);

    window.localStorage.setItem(
      "cuentas-claras-theme",
      nextTheme,
    );
  };

const handleCheckConnection = async () => {
  if (status === "loading") return;

  setStatus("loading");
  setData(null);

  try {
    // Invocamos el comando real de Rust
    const response = await invoke<ConnectionData>("ping_servidor");

    setData(response);
    setStatus(response.estado === "ok" ? "connected" : "error");
  } catch (error) {
    console.error("Error al conectar con el backend de Rust:", error);
    setStatus("error");
    setData({
      estado: "error",
      mensaje: String(error), // Muestra el mensaje de error amigable que devuelve tu Rust .map_err()
      version: "0.1.0",
      timestamp: "",
      latencia_ms: 0,
    });
  } finally {
    setLastCheck(
      new Intl.DateTimeFormat("es-VE", {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      }).format(new Date())
    );
  }
};
  const stateContent = {
    disconnected: {
      eyebrow:
        "SISTEMA EN ESPERA",

      title:
        "Listo para verificar",

      description:
        "Inicia una comprobación para establecer comunicación con el servidor.",
    },

    loading: {
      eyebrow:
        "ESTABLECIENDO ENLACE",

      title:
        "Conectando sistemas",

      description:
        "La solicitud está recorriendo la arquitectura de Cuentas Claras.",
    },

    connected: {
      eyebrow:
        "CONEXIÓN ESTABLECIDA",

      title:
        "Todo conectado",

      description:
        "La aplicación y el servidor se están comunicando correctamente.",
    },

    error: {
      eyebrow:
        "CONEXIÓN INTERRUMPIDA",

      title:
        "No pudimos conectar",

      description:
        "El servidor no respondió correctamente a la solicitud.",
    },
  };

  const content =
    stateContent[status];

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

          <div className="flex items-center gap-2.5">
            <div className="status-pill hidden items-center gap-2 rounded-full px-3 py-2 text-[10px] uppercase tracking-[0.14em] sm:flex">
              <span
                className={`h-1.5 w-1.5 rounded-full ${
                  status ===
                  "connected"
                    ? "bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.8)]"
                    : status ===
                        "loading"
                      ? "animate-pulse bg-cyan-400"
                      : status ===
                          "error"
                        ? "bg-red-400"
                        : "standby-dot"
                }`}
              />

              {status ===
              "connected"
                ? "Online"
                : status ===
                    "loading"
                  ? "Connecting"
                  : status ===
                      "error"
                    ? "Error"
                    : "Standby"}
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

        {/* MAIN CONTENT */}

        <section className="flex flex-1 items-center py-12 lg:py-8">
          <div className="grid w-full gap-12 lg:grid-cols-[0.78fr_1.22fr] lg:items-center lg:gap-16">
            {/* LEFT */}

            <div className="mx-auto w-full max-w-xl lg:mx-0">
              <div className="mb-7 flex items-center gap-3">
                <span className="eyebrow-line h-px w-7" />

                <span className="eyebrow-text text-[10px] font-semibold uppercase tracking-[0.28em]">
                  {
                    content.eyebrow
                  }
                </span>
              </div>

              <h1 className="hero-title max-w-xl text-[clamp(3rem,6vw,6rem)] font-medium leading-[0.9] tracking-[-0.065em]">
                Claridad

                <span className="hero-gradient block pb-2">
                  conectada.
                </span>
              </h1>

              <p className="secondary-text mt-7 max-w-lg text-sm leading-7 sm:text-base">
                Una infraestructura
                que conecta interfaz,
                aplicación local y
                servidor en una sola
                experiencia visual.
              </p>

              {/* BUTTON */}

              <div className="mt-10">
                <button
                  type="button"
                  disabled={
                    status ===
                    "loading"
                  }
                  onClick={
                    handleCheckConnection
                  }
                  className="tech-button group relative flex w-full max-w-sm items-center justify-between overflow-hidden rounded-2xl px-5 py-4 text-left disabled:cursor-not-allowed"
                >
                  <div className="tech-button-scan" />

                  <div className="relative z-10">
                    <p className="button-eyebrow text-[10px] uppercase tracking-[0.22em]">
                      Acción del
                      sistema
                    </p>

                    <p className="primary-text mt-1 text-sm font-semibold">
                      {status ===
                      "loading"
                        ? "Estableciendo conexión..."
                        : status ===
                            "connected"
                          ? "Verificar nuevamente"
                          : "Verificar conexión"}
                    </p>
                  </div>

                  <div className="button-arrow relative z-10 flex h-10 w-10 items-center justify-center rounded-xl">
                    {status ===
                    "loading" ? (
                      <div className="loading-spinner h-4 w-4 animate-spin rounded-full" />
                    ) : (
                      <span className="primary-text transition duration-300 group-hover:translate-x-1">
                        →
                      </span>
                    )}
                  </div>
                </button>
              </div>

              {/* STACK */}

              <div className="secondary-soft mt-7 flex flex-wrap items-center gap-4 text-[10px] uppercase tracking-[0.14em]">
                <span className="inline-flex items-center gap-2">
                  <IOSModuleIcon className="h-5 w-5" />

                  Interface
                </span>

                <span>
                  /
                </span>

                <span>
                  Rust Bridge
                </span>

                <span>
                  /
                </span>

                <span>
                  FastAPI Server
                </span>
              </div>
            </div>

            {/* RIGHT */}

            <div className="relative mx-auto w-full max-w-3xl">
              <div
                className="panel-backlight absolute left-1/2 top-1/2 h-[420px] w-[420px] -translate-x-1/2 -translate-y-1/2 rounded-full blur-[100px]"
                aria-hidden="true"
              />

              <div className="system-card relative overflow-hidden rounded-[32px] p-5 backdrop-blur-2xl sm:p-7 lg:p-9">
                <div className="system-card-shine" />

                {/* CARD HEADER */}

                <div
                  className="relative z-10 flex items-start justify-between gap-5"
                  role="status"
                  aria-live="polite"
                >
                  <div>
                    <p className="secondary-soft text-[10px] font-medium uppercase tracking-[0.24em]">
                      Core connection
                    </p>

                    <h2 className="primary-text mt-2 text-xl font-semibold tracking-[-0.02em] sm:text-2xl">
                      {
                        content.title
                      }
                    </h2>

                    <p className="secondary-text mt-2 max-w-md text-xs leading-6 sm:text-sm">
                      {
                        content.description
                      }
                    </p>
                  </div>

                  <div className="cc-badge flex items-center gap-3 rounded-2xl px-3 py-2">
                    <IOSModuleIcon className="h-8 w-8" />

                    <span className="secondary-soft font-mono text-[9px] uppercase tracking-[0.15em]">
                      CC-01
                    </span>
                  </div>
                </div>

                {/* ORB */}

                <div className="relative z-10 flex min-h-[320px] items-center justify-center py-8 sm:min-h-[370px]">
                  <StatusOrb
                    status={status}
                    latency={
                      data?.latencia_ms
                    }
                  />
                </div>

                {/* FLOW */}

                <SystemFlow
                  status={status}
                />

                {/* DATA */}

                <div className="relative z-10 mt-9 grid gap-3 sm:grid-cols-[1.5fr_0.7fr_0.7fr]">
                  <DataItem
                    label="Server message"
                    value={
                      data?.mensaje ??
                      "Sin respuesta"
                    }
                  />

                  <DataItem
                    label="Version"
                    value={
                      data?.version ??
                      "--"
                    }
                  />

                  <DataItem
                    label="Latency"
                    value={
                      data
                        ? `${data.latencia_ms} ms`
                        : "--"
                    }
                    accent={
                      status ===
                      "connected"
                    }
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
          </div>
        </section>

        {/* FOOTER */}

        <footer className="footer-border secondary-soft flex flex-col gap-2 border-t py-5 text-[10px] uppercase tracking-[0.15em] sm:flex-row sm:items-center sm:justify-between">
          <span>
            Cuentas Claras ·
            Digital
            Infrastructure
          </span>

          <span>
            Next.js · Tauri ·
            FastAPI
          </span>
        </footer>
      </div>
    </main>
  );
}