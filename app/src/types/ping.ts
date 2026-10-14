/**
 * Resultado del comando Tauri `ping_servidor`.
 * - estado, mensaje, version y timestamp vienen del servidor (GET /ping).
 * - latencia_ms la mide Rust alrededor de la petición HTTP.
 */
export interface PingResponse {
  estado: string;
  mensaje: string;
  version: string;
  /** Fecha y hora en formato ISO 8601 (UTC). */
  timestamp: string;
  /** Latencia de la petición HTTP, en milisegundos. */
  latencia_ms: number;
}

export function isPingResponse(value: unknown): value is PingResponse {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const response = value as Record<string, unknown>;

  return (
    typeof response.estado === "string" &&
    typeof response.mensaje === "string" &&
    typeof response.version === "string" &&
    typeof response.timestamp === "string" &&
    !Number.isNaN(Date.parse(response.timestamp)) &&
    typeof response.latencia_ms === "number" &&
    Number.isFinite(response.latencia_ms) &&
    response.latencia_ms >= 0
  );
}

/**
 * Estado de la verificación de conexión.
 * Cada variante solo contiene los datos que tienen sentido en ese estado.
 */
export type ConnectionState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; data: PingResponse }
  | { status: "error"; message: string };