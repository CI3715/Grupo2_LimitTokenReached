"use client";

import Link from "next/link";
import { useState } from "react";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { registrarUsuario } from "@/services/autenticacion";
import { GlobalForm, FormField } from "@/components/forms";
import { registerSchema, type RegisterFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";
import type { UseFormReturn } from "react-hook-form";

export default function RegistroPage() {
  const [mensaje, setMensaje] = useState("");
  const [registrado, setRegistrado] = useState(false);
  async function enviar(datos: RegisterFormValues, form: UseFormReturn<RegisterFormValues>) {
    setMensaje("");
    try {
      await registrarUsuario(datos);
      setRegistrado(true);
    } catch (fallo: unknown) {
      const validacion = applyBackendValidationError(fallo, form.setError);
      if (!validacion) {
        setMensaje(typeof fallo === "string" ? fallo : "No se pudo completar el registro. Inténtalo de nuevo.");
      }
    }
  }

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="auth" />
        <main className="accounts-workspace">
          <section className="auth-card" aria-labelledby="register-title">
            <p className="breadcrumb">Cuenta</p>
            <h1 id="register-title">Crear cuenta</h1>
            {registrado ? (
              <>
                <p role="status">Cuenta creada. Ya puedes iniciar sesión.</p>
                <Link className="account-primary auth-link-button" href="/login/">
                  Ir a iniciar sesión
                </Link>
              </>
            ) : (
              <>
                <p>El registro es opcional; tus finanzas permanecen en el dispositivo.</p>
                <GlobalForm<RegisterFormValues>
                  className="auth-form"
                  schema={registerSchema}
                  defaultValues={{
                    nombre_usuario: "",
                    nombre_visible: "",
                    correo: "",
                    contrasena: "",
                  }}
                  onSubmit={enviar}
                >
                  {(form) => (
                    <>
                      <FormField<RegisterFormValues> name="nombre_usuario" label="Nombre de usuario">
                        {(props) => <input {...props} autoComplete="username" {...form.register("nombre_usuario")} />}
                      </FormField>
                      <FormField<RegisterFormValues> name="nombre_visible" label="Nombre visible">
                        {(props) => <input {...props} autoComplete="name" {...form.register("nombre_visible")} />}
                      </FormField>
                      <FormField<RegisterFormValues> name="correo" label="Correo">
                        {(props) => <input {...props} type="email" autoComplete="email" {...form.register("correo")} />}
                      </FormField>
                      <FormField<RegisterFormValues> name="contrasena" label="Contraseña (mínimo 10 caracteres)">
                        {(props) => <input {...props} type="password" autoComplete="new-password" {...form.register("contrasena")} />}
                      </FormField>
                  {mensaje && (
                    <p className="auth-error" role="alert">
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
            )}
          </section>
        </main>
      </div>
    </div>
  );
}
