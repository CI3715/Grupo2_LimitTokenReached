"use client";

import { useId, type InputHTMLAttributes, type ReactNode } from "react";
import { useFormContext, type FieldValues, type Path } from "react-hook-form";

type Props<TValues extends FieldValues> = {
  name: Path<TValues>;
  label: string;
  help?: ReactNode;
  children: (props: {
    id: string;
    "aria-invalid": boolean;
    "aria-describedby": string | undefined;
  }) => ReactNode;
  className?: string;
};

export function FormField<TValues extends FieldValues>({
  name,
  label,
  help,
  children,
  className,
}: Props<TValues>) {
  const id = useId();
  const { formState } = useFormContext<TValues>();
  const error = formState.errors[name];
  const errorId = `${id}-error`;
  const helpId = help ? `${id}-help` : undefined;
  const describedBy = [helpId, error ? errorId : undefined].filter(Boolean).join(" ") || undefined;

  return (
    <label className={`form-field${className ? ` ${className}` : ""}`} htmlFor={id}>
      {label}
      {children({
        id,
        "aria-invalid": Boolean(error),
        "aria-describedby": describedBy,
      })}
      {help && <small id={helpId}>{help}</small>}
      {error?.message && (
        <small id={errorId} className="account-error" role="alert">
          {String(error.message)}
        </small>
      )}
    </label>
  );
}

export type FormInputProps = InputHTMLAttributes<HTMLInputElement>;
