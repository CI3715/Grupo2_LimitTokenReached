"use client";

import { useFormContext, type FieldValues } from "react-hook-form";

export function FormError<TValues extends FieldValues>() {
  const { formState } = useFormContext<TValues>();
  const message = formState.errors.root?.server?.message;
  return message ? <p role="alert">{String(message)}</p> : null;
}
