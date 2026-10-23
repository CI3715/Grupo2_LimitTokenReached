import { invoke } from "@tauri-apps/api/core";
import type { MovimientoDTO } from "@/types/movements";

export async function listMovements(accountId: string): Promise<MovimientoDTO[]> {
  return invoke<MovimientoDTO[]>("listar_historial_movimientos", {
    cuentaId: accountId,
  });
}
