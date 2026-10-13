export interface PingResponse {
  estado: string;
  mensaje: string;
  version: string;
  timestamp: string;
  latencia_ms: number;
}
export type ConnectionState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; data: PingResponse }
  | { status: "error"; message: string };
export function isPingResponse(value: unknown): value is PingResponse {
  if (typeof value !== "object" || value === null) return false;
  const data = value as Record<string, unknown>;
  return (
    data.estado === "ok" &&
    typeof data.mensaje === "string" &&
    typeof data.version === "string" &&
    typeof data.timestamp === "string" &&
    !Number.isNaN(Date.parse(data.timestamp)) &&
    typeof data.latencia_ms === "number" &&
    Number.isFinite(data.latencia_ms) &&
    data.latencia_ms >= 0
  );
}
