import { invoke } from "@tauri-apps/api/core";

export function deleteRate(tasaId: string): Promise<void> {
  return invoke("eliminar_tasa", {
    datos: { tasa_id: tasaId },
  });
}
