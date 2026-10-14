import { invoke } from "@tauri-apps/api/core";

import {
  isPingResponse,
  type PingResponse,
} from "@/types/ping";

export async function pingServer(): Promise<PingResponse> {
  const response: unknown = await invoke("ping_servidor");

  if (!isPingResponse(response)) {
    throw new Error("El servidor devolvió una respuesta de ping inválida.");
  }

  return response;
}
