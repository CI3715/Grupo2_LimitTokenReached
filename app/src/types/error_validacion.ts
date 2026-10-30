export interface ErrorValidacionDTO {
  campo: string;
  mensaje: string;
}

export function parseErrorValidacion(
  value: unknown,
): ErrorValidacionDTO | null {
  let candidate = value;

  if (candidate instanceof Error) {
    candidate = candidate.message;
  }

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
    return {
      campo: candidate.campo,
      mensaje: candidate.mensaje,
    };
  }

  return null;
}
