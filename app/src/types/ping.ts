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

/**
 * Estado de la verificación de conexión.
 * Cada variante solo contiene los datos que tienen sentido en ese estado.
 */
export type ConnectionState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; data: PingResponse }
  | { status: "error"; message: string };