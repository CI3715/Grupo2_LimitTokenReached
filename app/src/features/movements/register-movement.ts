"use client";

import { createElement } from "react";
import { MovementForm, type MovementFormProps } from "@/components/movements/movement-form";
import { movementError, registerMovement } from "@/services/registerMovement";

export async function registerMovementFromForm(
  request: Parameters<MovementFormProps["onRegister"]>[0],
) {
  try {
    return await registerMovement(request);
  } catch (error: unknown) {
    throw movementError(error);
  }
}

export function RegisterMovement(props: Omit<MovementFormProps, "onRegister">) {
  return createElement(MovementForm, {
    ...props,
    key: props.account.id,
    onRegister: registerMovementFromForm,
  });
}
