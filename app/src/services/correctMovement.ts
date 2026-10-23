import { invoke } from "@tauri-apps/api/core";
import type { CorregirMovimientoRequest, MovimientoDTO } from "@/types/movements";

export async function correctMovement(
  request: CorregirMovimientoRequest,
): Promise<MovimientoDTO> {
  const result = await invoke<MovimientoDTO>("corregir_movimiento", {
    datos: request,
  });

  if (
    !result ||
    typeof result.id !== "string" ||
    result.es_correccion !== true ||
    result.movimiento_original_id !== request.movimiento_id ||
    typeof result.monto !== "string"
  ) {
    throw {
      mensaje:
        "No se pudo confirmar la corrección. Revisa los movimientos de la cuenta antes de volver a intentarlo.",
      verificarRegistro: true,
    };
  }

  return result;
}
