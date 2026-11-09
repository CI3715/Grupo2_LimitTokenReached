"use client";

import { useFormContext, type FieldValues, type Path } from "react-hook-form";

type Option = { value: string; label: string };

type Props<TValues extends FieldValues> = {
  name: Path<TValues>;
  options: Option[];
  disabled?: boolean;
  id: string;
  "aria-invalid"?: boolean;
  "aria-describedby"?: string;
};

export function FormSelect<TValues extends FieldValues>({
  name,
  options,
  disabled,
  id,
  ...aria
}: Props<TValues>) {
  const { register } = useFormContext<TValues>();
  return (
    <select id={id} {...register(name)} disabled={disabled} {...aria}>
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}
