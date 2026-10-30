import { invoke } from "@tauri-apps/api/core";
import type { TasaDTO } from "@/types/tasas";

export function listRateHistory(date: string): Promise<TasaDTO[]> {
  return invoke<TasaDTO[]>("listar_historial_tasas", { fecha: date });
}
