"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import {
  FormProvider,
  useForm,
  type DefaultValues,
  type FieldValues,
  type Resolver,
  type UseFormReturn,
} from "react-hook-form";
import { z } from "zod";
import type { ReactNode } from "react";

type Props<TValues extends FieldValues> = {
  schema: z.ZodType<TValues>;
  defaultValues: DefaultValues<TValues>;
  onSubmit: (values: TValues, form: UseFormReturn<TValues>) => void | Promise<void>;
  children: (form: UseFormReturn<TValues>) => ReactNode;
  className?: string;
};

export function GlobalForm<TValues extends FieldValues>({
  schema,
  defaultValues,
  onSubmit,
  children,
  className,
}: Props<TValues>) {
  const form = useForm<TValues>({
    resolver: zodResolver(schema as never) as Resolver<TValues>,
    defaultValues,
    mode: "onSubmit",
  });

  return (
    <FormProvider {...form}>
      <form
        className={className}
        onSubmit={form.handleSubmit((values) => onSubmit(values, form))}
        aria-busy={form.formState.isSubmitting}
      >
        {form.formState.errors.root?.server?.message && (
          <p role="alert">{form.formState.errors.root.server.message}</p>
        )}
        {children(form)}
      </form>
    </FormProvider>
  );
}
