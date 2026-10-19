"use client";

import { MovementForm, type MovementFormProps } from "@/components/movements/movement-form";
import { movementError, registerMovement } from "@/services/registerMovement";

// La vista de detalle aporta cuenta, moneda, catálogo y actualización tras el éxito.
export function RegisterMovement(props: Omit<MovementFormProps, "onRegister">) {
  return <MovementForm {...props} key={props.account.id} onRegister={async (request) => {
    try {
      return await registerMovement(request);
    } catch (error: unknown) {
      throw movementError(error);
    }
  }} />;
}
