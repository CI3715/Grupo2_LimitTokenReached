import { invoke } from "@tauri-apps/api/core";
import { expenseExceedsBalance } from "@/types/movements";
import type {
  MovimientoDTO,
  MovementFormError,
  RegistrarMovimientoRequest,
} from "@/types/movements";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function movementError(error: unknown): MovementFormError {
  let value = error;
  if (typeof value === "string") {
    try { value = JSON.parse(value); } catch { /* Mensaje simple de Tauri. */ }
  }
  if (isRecord(value) && typeof value.mensaje === "string") {
    return {
      mensaje: value.mensaje,
      campo: typeof value.campo === "string" ? value.campo : undefined,
      verificarRegistro: value.verificarRegistro === true,
    };
  }
  return {
    mensaje: typeof value === "string" && value.trim()
      ? value
      : "No se pudo registrar el movimiento. Intenta nuevamente.",
  };
}

export async function registerMovement(
  request: RegistrarMovimientoRequest,
): Promise<MovimientoDTO> {
  // Convención de los comandos actuales (crear_cuenta): argumento Rust `datos`.
  const result = await invoke<unknown>("registrar_movimiento", { datos: request });
  if (
    !isRecord(result) ||
    typeof result.id !== "string" || !result.id.trim() ||
    result.cuenta_id !== request.cuenta_id ||
    result.tipo !== request.tipo ||
    typeof result.monto !== "string" || !/^\d+(?:\.\d+)?$/.test(result.monto) ||
    expenseExceedsBalance(result.monto, request.monto) ||
    expenseExceedsBalance(request.monto, result.monto) ||
    result.fecha !== request.fecha ||
    result.categoria_id !== request.categoria_id ||
    result.nota !== request.nota ||
    result.es_correccion !== false ||
    result.movimiento_original_id !== null
  ) {
    throw {
      mensaje: "No se pudo confirmar la respuesta del registro. Revisa los movimientos de la cuenta antes de volver a intentarlo.",
      verificarRegistro: true,
    } satisfies MovementFormError;
  }
  return result as unknown as MovimientoDTO;
}
