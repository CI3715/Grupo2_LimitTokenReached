import { invoke, isTauri } from "@tauri-apps/api/core";
import { isPingResponse, type PingResponse } from "@/types/ping";
export async function pingServer(): Promise<PingResponse> {
  if (!isTauri())
    throw new Error(
      "Abre Cuentas Claras en la aplicación de escritorio para comprobar la conexión. Esta vista del navegador no puede consultar el servidor.",
    );
  let response: unknown;
  try {
    response = await invoke("ping_servidor");
  } catch {
    throw new Error(
      "No pudimos conectar con el servidor. Comprueba que esté encendido y que esta versión de la app incluya la comprobación de conexión.",
    );
  }
  if (!isPingResponse(response))
    throw new Error(
      "No pudimos confirmar el estado del servidor. Inténtalo de nuevo.",
    );
  return response;
}
