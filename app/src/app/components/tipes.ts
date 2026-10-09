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
  timestamp: string;
  latencia_ms: number;
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

export type { ConnectionStatus, Theme, ConnectionData, Particle };