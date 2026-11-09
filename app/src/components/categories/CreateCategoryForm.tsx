"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { FormField } from "@/components/forms";
import { createCategorySchema, type CreateCategoryFormValues } from "@/types/schemas";
import type { CategoriaDTO } from "@/types/movements";

type Props = {
  defaultType: "Ingreso" | "Gasto";
  disabled?: boolean;
  onCreate: (values: CreateCategoryFormValues) => Promise<CategoriaDTO>;
  onCreated: (category: CategoriaDTO) => void;
};

export function CreateCategoryForm({
  defaultType,
  disabled = false,
  onCreate,
  onCreated,
}: Props) {
  const form = useForm<CreateCategoryFormValues>({
    resolver: zodResolver(createCategorySchema),
    defaultValues: { nombre: "", tipo: defaultType },
  });

  async function submit(values: CreateCategoryFormValues) {
    try {
      const category = await onCreate(values);
      form.reset({ nombre: "", tipo: category.tipo as "Ingreso" | "Gasto" });
      onCreated(category);
    } catch (error: unknown) {
      const failure = error as { mensaje?: unknown };
      form.setError("root.server", {
        type: "server",
        message: typeof failure.mensaje === "string" ? failure.mensaje : "No se pudo crear la categoría.",
      });
    }
  }

  return (
    <div>
      <FormField<CreateCategoryFormValues> name="nombre" label="Nombre de la nueva categoría">
        {(props) => <input {...props} maxLength={80} disabled={disabled || form.formState.isSubmitting} {...form.register("nombre")} />}
      </FormField>
      <FormField<CreateCategoryFormValues> name="tipo" label="Tipo">
        {(props) => (
          <select {...props} disabled={disabled || form.formState.isSubmitting} {...form.register("tipo")}>
            <option value="Gasto">Gasto</option>
            <option value="Ingreso">Ingreso</option>
          </select>
        )}
      </FormField>
      {form.formState.errors.root?.server?.message && (
        <small className="movement-error" role="alert">{String(form.formState.errors.root.server.message)}</small>
      )}
      <button
        type="button"
        className="account-secondary"
        disabled={disabled || form.formState.isSubmitting}
        onClick={() => void form.handleSubmit(submit)()}
      >
        {form.formState.isSubmitting ? "Guardando…" : "Guardar categoría"}
      </button>
    </div>
  );
}
