import type { UseFormReturn } from "react-hook-form";

import { iniciarSesion } from "@/services/autenticacion";
import type { UsuarioDTO } from "@/types/autenticacion";
import type { LoginFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";

export async function iniciarSesionDesdeFormulario(
  datos: LoginFormValues,
  form: UseFormReturn<LoginFormValues>,
  mostrarMensaje: (mensaje: string) => void,
  alIniciarSesion: (usuario: UsuarioDTO) => void,
) {
  try {
    const usuario = await iniciarSesion(datos);
    alIniciarSesion(usuario);
  } catch (fallo: unknown) {
    const validacion = applyBackendValidationError(fallo, form.setError);
    if (!validacion) {
      mostrarMensaje(
        typeof fallo === "string"
          ? fallo
          : "No se pudo iniciar sesión. Inténtalo de nuevo.",
      );
    }
  }
}
