import { correctMovement } from "@/services/correctMovement";
import { movementError } from "@/services/registerMovement";
import type {
  CorregirMovimientoRequest,
  MovimientoDTO,
  MovementFormError,
} from "@/types/movements";

export async function correctMovementFromForm(
  request: CorregirMovimientoRequest,
): Promise<MovimientoDTO> {
  try {
    return await correctMovement(request);
  } catch (error: unknown) {
    throw movementError(error) satisfies MovementFormError;
  }
}
