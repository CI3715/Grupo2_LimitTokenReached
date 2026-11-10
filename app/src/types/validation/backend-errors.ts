import type { FieldValues, Path, UseFormSetError } from "react-hook-form";
import { parseErrorValidacion, type ErrorValidacionDTO } from "@/types/error_validacion";

export function applyBackendValidationError<TValues extends FieldValues>(
  error: unknown,
  setError: UseFormSetError<TValues>,
): ErrorValidacionDTO | null {
  const validation = parseErrorValidacion(error);
  if (!validation) return null;

  if (validation.campo) {
    setError(validation.campo as Path<TValues>, {
      type: "server",
      message: validation.mensaje,
    });
  } else {
    setError("root.server", {
      type: "server",
      message: validation.mensaje,
    });
  }
  return validation;
}
