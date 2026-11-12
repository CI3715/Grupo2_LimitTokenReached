import type { UseFormReturn } from "react-hook-form";

import { registrarUsuario } from "@/services/autenticacion";
import type { RegisterFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";

export async function registrarDesdeFormulario(
  datos: RegisterFormValues,
  form: UseFormReturn<RegisterFormValues>,
  mostrarMensaje: (mensaje: string) => void,
  alRegistrar: () => void,
) {
  try {
    await registrarUsuario(datos);
    alRegistrar();
  } catch (fallo: unknown) {
    const validacion = applyBackendValidationError(fallo, form.setError);
    if (!validacion) {
      mostrarMensaje(
        typeof fallo === "string"
          ? fallo
          : "No se pudo completar el registro. Inténtalo de nuevo.",
      );
    }
  }
}
