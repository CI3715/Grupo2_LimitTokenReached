import type { FieldValues, Path, UseFormSetError } from "react-hook-form";
import type { ErrorValidacionDTO } from "./error_validacion";

function parseErrorValidacion(value: unknown): ErrorValidacionDTO | null {
  let candidate = value;
  if (candidate instanceof Error) candidate = candidate.message;
  if (typeof candidate === "string") {
    try {
      candidate = JSON.parse(candidate) as unknown;
    } catch {
      return null;
    }
  }
  if (
    typeof candidate === "object" &&
    candidate !== null &&
    "campo" in candidate &&
    "mensaje" in candidate &&
    typeof candidate.campo === "string" &&
    typeof candidate.mensaje === "string"
  ) {
    return { campo: candidate.campo, mensaje: candidate.mensaje };
  }
  return null;
}

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
