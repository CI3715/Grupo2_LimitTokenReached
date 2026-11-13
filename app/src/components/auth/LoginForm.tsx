"use client";

import Link from "next/link";
import type { UseFormReturn } from "react-hook-form";

import { GlobalForm, FormField } from "@/components/forms";
import { loginSchema, type LoginFormValues } from "@/types/schemas";
import "./auth.css";

type LoginFormProps = {
  mensaje: string;
  onSubmit: (
    datos: LoginFormValues,
    form: UseFormReturn<LoginFormValues>,
  ) => void | Promise<void>;
};

export function LoginForm({ mensaje, onSubmit }: LoginFormProps) {
  return (
    <section className="auth-card" aria-labelledby="login-title">
      <p className="breadcrumb">Cuenta</p>
      <h1 id="login-title">Iniciar sesión</h1>
      <p>Identifícate ante el servidor para usar las funciones compartidas.</p>
      <GlobalForm<LoginFormValues>
        className="form-stack"
        schema={loginSchema}
        defaultValues={{ correo: "", contrasena: "" }}
        onSubmit={onSubmit}
      >
        {(form) => (
          <>
            <FormField<LoginFormValues> name="correo" label="Correo">
              {(props) => (
                <input
                  {...props}
                  type="email"
                  autoComplete="email"
                  {...form.register("correo")}
                />
              )}
            </FormField>
            <FormField<LoginFormValues> name="contrasena" label="Contraseña">
              {(props) => (
                <input
                  {...props}
                  type="password"
                  autoComplete="current-password"
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
              {form.formState.isSubmitting ? "Conectando…" : "Iniciar sesión"}
            </button>
          </>
        )}
      </GlobalForm>
      <p className="auth-footer">
        ¿No tienes cuenta? <Link href="/registro/">Regístrate</Link>
      </p>
      <p className="auth-note">
        Si el servidor no está disponible, tus datos financieros locales
        seguirán funcionando.
      </p>
    </section>
  );
}
