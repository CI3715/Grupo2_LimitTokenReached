"use client";

import Link from "next/link";
import { useState } from "react";
import type { UseFormReturn } from "react-hook-form";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { RegisterForm } from "@/components/auth/RegisterForm";
import { registrarDesdeFormulario } from "@/features/auth/register";
import type { RegisterFormValues } from "@/types/schemas";
import { useAuthSession } from "@/features/auth/useAuthSession";

export default function RegistroPage() {
  const [mensaje, setMensaje] = useState("");
  const [registrado, setRegistrado] = useState(false);
  const { usuario } = useAuthSession();
  function enviar(
    datos: RegisterFormValues,
    form: UseFormReturn<RegisterFormValues>,
  ) {
    setMensaje("");
    return registrarDesdeFormulario(datos, form, setMensaje, () =>
      setRegistrado(true),
    );
  }

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="auth" sesionActiva={Boolean(usuario)} />
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
                <RegisterForm mensaje={mensaje} onSubmit={enviar} />
              </>
            )}
          </section>
        </main>
      </div>
    </div>
  );
}
