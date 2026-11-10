"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { iniciarSesion } from "@/services/autenticacion";
import { GlobalForm, FormField } from "@/components/forms";
import { loginSchema, type LoginFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";
import type { UseFormReturn } from "react-hook-form";

export default function LoginPage() {
  const router = useRouter();
  const [mensaje, setMensaje] = useState("");
  async function enviar(datos: LoginFormValues, form: UseFormReturn<LoginFormValues>) {
    setMensaje("");
    try {
      await iniciarSesion(datos);
      window.dispatchEvent(new Event("auth-session-changed"));
      router.replace("/");
    } catch (fallo: unknown) {
      const validacion = applyBackendValidationError(fallo, form.setError);
      if (!validacion) {
        setMensaje(typeof fallo === "string" ? fallo : "No se pudo iniciar sesión. Inténtalo de nuevo.");
      }
    }
  }

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="auth" />
        <main className="accounts-workspace">
          <section className="auth-card" aria-labelledby="login-title">
            <p className="breadcrumb">Cuenta</p>
            <h1 id="login-title">Iniciar sesión</h1>
            <p>Identifícate ante el servidor para usar las funciones compartidas.</p>
            <GlobalForm<LoginFormValues>
              className="auth-form"
              schema={loginSchema}
              defaultValues={{ correo: "", contrasena: "" }}
              onSubmit={enviar}
            >
              {(form) => (
                <>
                  <FormField<LoginFormValues> name="correo" label="Correo">
                    {(props) => <input {...props} type="email" autoComplete="email" {...form.register("correo")} />}
                  </FormField>
                  <FormField<LoginFormValues> name="contrasena" label="Contraseña">
                    {(props) => <input {...props} type="password" autoComplete="current-password" {...form.register("contrasena")} />}
                  </FormField>
                  {mensaje && <p className="auth-error" role="alert">{mensaje}</p>}
                  <button className="account-primary" type="submit" disabled={form.formState.isSubmitting}>
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
        </main>
      </div>
    </div>
  );
}
