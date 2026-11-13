"use client";

import Link from "next/link";
import type { UseFormReturn } from "react-hook-form";

import { FormField, GlobalForm } from "@/components/forms";
import { registerSchema, type RegisterFormValues } from "@/types/schemas";
import "./auth.css";

type RegisterFormProps = {
  mensaje: string;
  onSubmit: (
    datos: RegisterFormValues,
    form: UseFormReturn<RegisterFormValues>,
  ) => void | Promise<void>;
};

export function RegisterForm({ mensaje, onSubmit }: RegisterFormProps) {
  return (
    <>
      <p>El registro es opcional; tus finanzas permanecen en el dispositivo.</p>
      <GlobalForm<RegisterFormValues>
        className="form-stack"
        schema={registerSchema}
        defaultValues={{
          nombre_usuario: "",
          nombre_visible: "",
          correo: "",
          contrasena: "",
        }}
        onSubmit={onSubmit}
      >
        {(form) => (
          <>
            <FormField<RegisterFormValues>
              name="nombre_usuario"
              label="Nombre de usuario"
            >
              {(props) => (
                <input
                  {...props}
                  autoComplete="username"
                  {...form.register("nombre_usuario")}
                />
              )}
            </FormField>
            <FormField<RegisterFormValues>
              name="nombre_visible"
              label="Nombre visible"
            >
              {(props) => (
                <input
                  {...props}
                  autoComplete="name"
                  {...form.register("nombre_visible")}
                />
              )}
            </FormField>
            <FormField<RegisterFormValues> name="correo" label="Correo">
              {(props) => (
                <input
                  {...props}
                  type="email"
                  autoComplete="email"
                  {...form.register("correo")}
                />
              )}
            </FormField>
            <FormField<RegisterFormValues>
              name="contrasena"
              label="Contraseña (mínimo 10 caracteres)"
            >
              {(props) => (
                <input
                  {...props}
                  type="password"
                  autoComplete="new-password"
                  {...form.register("contrasena")}
                />
              )}
            </FormField>
            {mensaje && (
              <p className="form-error" role="alert">
                {mensaje}
              </p>
            )}
            <button
              className="account-primary"
              type="submit"
              disabled={form.formState.isSubmitting}
            >
              {form.formState.isSubmitting ? "Registrando…" : "Crear cuenta"}
            </button>
          </>
        )}
      </GlobalForm>
      <p className="auth-footer">
        ¿Ya tienes cuenta? <Link href="/login/">Inicia sesión</Link>
      </p>
    </>
  );
}
