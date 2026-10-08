"use client";

import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
} from "react";

type ConnectionStatus =
  | "disconnected"
  | "loading"
  | "connected"
  | "error";

type Theme = "dark" | "light";

type ConnectionData = {
  estado: string;
  mensaje: string;
  version: string;
  latencia: number;
};

type Particle = {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  depth: number;
  phase: number;
};

/* =========================================================
   IOS / VISIONOS STYLE ICON
========================================================= */

function IOSModuleIcon({
  className = "",
}: {
  className?: string;
}) {
  return (
    <div
      className={`ios-module-icon ${className}`}
      aria-hidden="true"
    >
      <span className="ios-module-bar ios-module-bar-top" />
      <span className="ios-module-bar ios-module-bar-bottom" />
    </div>
  );
}

/* =========================================================
   THEME ICONS
========================================================= */

function SunIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="h-[17px] w-[17px]"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="3.5" />

      <path d="M12 2.5v2" />
      <path d="M12 19.5v2" />

      <path d="M4.5 4.5l1.4 1.4" />
      <path d="M18.1 18.1l1.4 1.4" />

      <path d="M2.5 12h2" />
      <path d="M19.5 12h2" />

      <path d="M4.5 19.5l1.4-1.4" />
      <path d="M18.1 5.9l1.4-1.4" />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="h-[17px] w-[17px]"
      aria-hidden="true"
    >
      <path d="M20.5 15.4A8.5 8.5 0 0 1 8.6 3.5 8.5 8.5 0 1 0 20.5 15.4Z" />
    </svg>
  );
}

function ThemeToggle({
  theme,
  onToggle,
}: {
  theme: Theme;
  onToggle: () => void;
}) {
  const nextTheme =
    theme === "dark"
      ? "claro"
      : "oscuro";

  return (
    <button
      type="button"
      onClick={onToggle}
      className="theme-toggle"
      aria-label={`Cambiar a modo ${nextTheme}`}
      title={`Cambiar a modo ${nextTheme}`}
    >
      <span className="theme-toggle-glow" />

      <span className="relative z-10">
        {theme === "dark" ? (
          <SunIcon />
        ) : (
          <MoonIcon />
        )}
      </span>
    </button>
  );
}

/* =========================================================
   CANVAS BACKGROUND
========================================================= */

function NetworkBackground({
  status,
  theme,
}: {
  status: ConnectionStatus;
  theme: Theme;
}) {
  const canvasRef =
    useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas =
      canvasRef.current;

    if (!canvas) return;

    const ctx =
      canvas.getContext("2d");

    if (!ctx) return;

    let frame = 0;

    let animationFrame = 0;

    let width =
      window.innerWidth;

    let height =
      window.innerHeight;

    const pointer = {
      x: 0,
      y: 0,
    };

    const particles: Particle[] =
      [];

    const palette =
      theme === "dark"
        ? {
            grid: [
              82,
              144,
              255,
            ],
            particle: [
              111,
              177,
              255,
            ],
            connection: [
              92,
              159,
              255,
            ],
          }
        : {
            grid: [
              48,
              105,
              210,
            ],
            particle: [
              48,
              108,
              212,
            ],
            connection: [
              60,
              120,
              220,
            ],
          };

    const getParticleCount =
      () => {
        if (
          window.innerWidth <
          640
        ) {
          return 32;
        }

        if (
          window.innerWidth <
          1024
        ) {
          return 52;
        }

        return 82;
      };

    const createParticles =
      () => {
        particles.length = 0;

        const count =
          getParticleCount();

        for (
          let i = 0;
          i < count;
          i++
        ) {
          const depth =
            Math.random();

          particles.push({
            x:
              Math.random() *
              width,

            y:
              Math.random() *
              height,

            vx:
              (Math.random() -
                0.5) *
              (0.08 +
                depth * 0.16),

            vy:
              (Math.random() -
                0.5) *
              (0.08 +
                depth * 0.16),

            size:
              0.5 +
              depth * 1.3,

            depth,

            phase:
              Math.random() *
              Math.PI *
              2,
          });
        }
      };

    const resize = () => {
      const dpr = Math.min(
        window.devicePixelRatio ||
          1,
        2,
      );

      width =
        window.innerWidth;

      height =
        window.innerHeight;

      canvas.width =
        width * dpr;

      canvas.height =
        height * dpr;

      canvas.style.width =
        `${width}px`;

      canvas.style.height =
        `${height}px`;

      ctx.setTransform(
        dpr,
        0,
        0,
        dpr,
        0,
        0,
      );

      createParticles();
    };

    const handlePointerMove = (
      event: PointerEvent,
    ) => {
      pointer.x =
        event.clientX /
          width -
        0.5;

      pointer.y =
        event.clientY /
          height -
        0.5;
    };

    const drawPerspectiveGrid =
      () => {
        const horizon =
          height * 0.47;

        const bottom =
          height + 120;

        const center =
          width / 2 +
          pointer.x * 25;

        ctx.save();

        ctx.lineWidth = 0.7;

        for (
          let i = -10;
          i <= 10;
          i++
        ) {
          const bottomX =
            center +
            i *
              (width / 8);

          const gradient =
            ctx.createLinearGradient(
              center,
              horizon,
              bottomX,
              bottom,
            );

          const [
            r,
            g,
            b,
          ] = palette.grid;

          gradient.addColorStop(
            0,
            `rgba(${r}, ${g}, ${b}, 0)`,
          );

          gradient.addColorStop(
            1,
            `rgba(${r}, ${g}, ${b}, ${
              theme === "dark"
                ? 0.1
                : 0.07
            })`,
          );

          ctx.strokeStyle =
            gradient;

          ctx.beginPath();

          ctx.moveTo(
            center,
            horizon,
          );

          ctx.lineTo(
            bottomX,
            bottom,
          );

          ctx.stroke();
        }

        const rows = 12;

        for (
          let i = 0;
          i < rows;
          i++
        ) {
          const progress =
            i / rows;

          const eased =
            progress *
            progress;

          const y =
            horizon +
            eased *
              (bottom -
                horizon);

          const [
            r,
            g,
            b,
          ] = palette.grid;

          const opacity =
            theme === "dark"
              ? 0.015 +
                progress *
                  0.055
              : 0.012 +
                progress *
                  0.035;

          ctx.strokeStyle =
            `rgba(${r}, ${g}, ${b}, ${opacity})`;

          ctx.beginPath();

          ctx.moveTo(
            0,
            y,
          );

          ctx.lineTo(
            width,
            y,
          );

          ctx.stroke();
        }

        ctx.restore();
      };

    const drawParticles =
      () => {
        for (
          let i = 0;
          i <
          particles.length;
          i++
        ) {
          const particle =
            particles[i];

          particle.x +=
            particle.vx;

          particle.y +=
            particle.vy;

          if (
            particle.x < -20
          ) {
            particle.x =
              width + 20;
          }

          if (
            particle.x >
            width + 20
          ) {
            particle.x = -20;
          }

          if (
            particle.y < -20
          ) {
            particle.y =
              height + 20;
          }

          if (
            particle.y >
            height + 20
          ) {
            particle.y = -20;
          }

          const parallaxX =
            pointer.x *
            (8 +
              particle.depth *
                18);

          const parallaxY =
            pointer.y *
            (8 +
              particle.depth *
                18);

          const x =
            particle.x +
            parallaxX;

          const y =
            particle.y +
            parallaxY;

          const pulse =
            0.7 +
            Math.sin(
              frame *
                0.012 +
                particle.phase,
            ) *
              0.3;

          let intensity =
            theme === "dark"
              ? 0.16
              : 0.15;

          if (
            status ===
            "loading"
          ) {
            intensity =
              theme === "dark"
                ? 0.3
                : 0.23;
          }

          if (
            status ===
            "connected"
          ) {
            intensity =
              theme === "dark"
                ? 0.38
                : 0.29;
          }

          const [
            pr,
            pg,
            pb,
          ] =
            palette.particle;

          ctx.beginPath();

          ctx.fillStyle =
            `rgba(${pr}, ${pg}, ${pb}, ${
              intensity *
              pulse *
              (0.45 +
                particle.depth)
            })`;

          ctx.arc(
            x,
            y,
            particle.size,
            0,
            Math.PI * 2,
          );

          ctx.fill();

          for (
            let j =
              i + 1;
            j <
            particles.length;
            j++
          ) {
            const other =
              particles[j];

            const ox =
              other.x +
              pointer.x *
                (8 +
                  other.depth *
                    18);

            const oy =
              other.y +
              pointer.y *
                (8 +
                  other.depth *
                    18);

            const dx =
              x - ox;

            const dy =
              y - oy;

            const distance =
              Math.sqrt(
                dx * dx +
                  dy * dy,
              );

            if (
              distance <
              120
            ) {
              const opacity =
                (1 -
                  distance /
                    120) *
                (theme ===
                "dark"
                  ? 0.055
                  : 0.035) *
                (status ===
                "connected"
                  ? 1.4
                  : 1);

              const [
                cr,
                cg,
                cb,
              ] =
                palette.connection;

              ctx.strokeStyle =
                `rgba(${cr}, ${cg}, ${cb}, ${opacity})`;

              ctx.lineWidth =
                0.65;

              ctx.beginPath();

              ctx.moveTo(
                x,
                y,
              );

              ctx.lineTo(
                ox,
                oy,
              );

              ctx.stroke();
            }
          }
        }
      };

    const draw = () => {
      frame += 1;

      ctx.clearRect(
        0,
        0,
        width,
        height,
      );

      drawPerspectiveGrid();

      drawParticles();

      animationFrame =
        requestAnimationFrame(
          draw,
        );
    };

    resize();

    draw();

    window.addEventListener(
      "resize",
      resize,
    );

    window.addEventListener(
      "pointermove",
      handlePointerMove,
    );

    return () => {
      cancelAnimationFrame(
        animationFrame,
      );

      window.removeEventListener(
        "resize",
        resize,
      );

      window.removeEventListener(
        "pointermove",
        handlePointerMove,
      );
    };
  }, [status, theme]);

  return (
    <canvas
      ref={canvasRef}
      className="pointer-events-none fixed inset-0 z-0"
      aria-hidden="true"
    />
  );
}

/* =========================================================
   CONNECTION ORB
========================================================= */

function StatusOrb({
  status,
  latency,
}: {
  status: ConnectionStatus;
  latency?: number;
}) {
  const duration = latency
    ? Math.max(
        1.4,
        Math.min(
          3.4,
          latency / 55,
        ),
      )
    : 2.4;

  const style = {
    "--orb-duration":
      `${duration}s`,
  } as CSSProperties;

  return (
    <div
      className={`orb orb-${status}`}
      style={style}
    >
      <div className="orb-aura" />

      <div className="orb-ring orb-ring-one" />

      <div className="orb-ring orb-ring-two" />

      <div className="orb-ring orb-ring-three" />

      <div className="orb-particle orb-particle-one" />

      <div className="orb-particle orb-particle-two" />

      <div className="orb-particle orb-particle-three" />

      <div className="orb-shell">
        <div className="orb-shell-highlight" />

        <div className="orb-core">
          <div className="orb-core-inner" />
        </div>
      </div>

      {status ===
        "loading" && (
        <>
          <div className="orb-wave orb-wave-one" />

          <div className="orb-wave orb-wave-two" />
        </>
      )}
    </div>
  );
}

/* =========================================================
   SYSTEM FLOW
========================================================= */

function FlowNode({
  title,
  subtitle,
  active,
}: {
  title: string;
  subtitle: string;
  active: boolean;
}) {
  return (
    <div className="relative z-10 flex min-w-0 flex-1 flex-col items-center">
      <div
        className={`flow-node ${
          active
            ? "flow-node-active"
            : ""
        }`}
      >
        <div className="flow-node-inner" />
      </div>

      <span className="flow-title mt-3 text-[11px] font-semibold tracking-[0.04em] sm:text-xs">
        {title}
      </span>

      <span className="flow-subtitle mt-1 hidden text-[9px] uppercase tracking-[0.18em] sm:block">
        {subtitle}
      </span>
    </div>
  );
}

function SystemFlow({
  status,
}: {
  status: ConnectionStatus;
}) {
  const firstActive =
    status === "loading" ||
    status === "connected";

  const secondActive =
    status === "connected";

  return (
    <div className="relative mx-auto mt-10 w-full max-w-xl">
      <div className="flow-base-line absolute left-[16.6%] right-[16.6%] top-[14px] h-px" />

      <div
        className={`signal-line ${
          firstActive
            ? "signal-line-active"
            : ""
        }`}
      />

      <div
        className={`signal-tracer ${
          status ===
          "loading"
            ? "signal-tracer-running"
            : status ===
                "connected"
              ? "signal-tracer-complete"
              : ""
        }`}
      />

      <div className="relative flex items-start justify-between">
        <FlowNode
          title="Next.js"
          subtitle="Interface"
          active={
            firstActive
          }
        />

        <FlowNode
          title="Tauri / Rust"
          subtitle="Bridge"
          active={
            firstActive
          }
        />

        <FlowNode
          title="FastAPI"
          subtitle="Server"
          active={
            secondActive
          }
        />
      </div>
    </div>
  );
}

/* =========================================================
   DATA CARDS
========================================================= */

function DataItem({
  label,
  value,
  accent = false,
}: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div className="data-item group relative overflow-hidden rounded-2xl p-4 transition duration-300">
      <div className="data-item-highlight absolute inset-x-0 top-0 h-px opacity-0 transition group-hover:opacity-100" />

      <p className="data-label text-[10px] font-medium uppercase tracking-[0.2em]">
        {label}
      </p>

      <p
        className={`mt-3 truncate font-mono text-sm font-medium ${
          accent
            ? "data-value-accent"
            : "data-value"
        }`}
      >
        {value}
      </p>
    </div>
  );
}

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

  const handleCheckConnection =
    () => {
      if (
        status === "loading"
      ) {
        return;
      }

      setStatus("loading");

      setData(null);

      /*
        TEMPORAL

        Esta respuesta sigue siendo simulada.

        Posteriormente este setTimeout será
        reemplazado por invoke("ping_servidor").

        IMPORTANTE:

        El frontend NO calcula la latencia.

        Rust entregará:
        - JSON del servidor
        - latencia

        Aquí solamente los mostramos.
      */

      window.setTimeout(
        () => {
          const response: ConnectionData =
            {
              estado: "ok",

              mensaje:
                "Servidor Cuentas Claras activo",

              version:
                "0.1.0",

              latencia:
                24,
            };

          setData(
            response,
          );

          setStatus(
            response.estado ===
              "ok"
              ? "connected"
              : "error",
          );

          setLastCheck(
            new Intl.DateTimeFormat(
              "es-VE",
              {
                hour:
                  "2-digit",

                minute:
                  "2-digit",

                second:
                  "2-digit",
              },
            ).format(
              new Date(),
            ),
          );
        },
        1600,
      );
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
                      data?.latencia
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
                        ? `${data.latencia} ms`
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