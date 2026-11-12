"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import type { UseFormReturn } from "react-hook-form";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { LoginForm } from "@/components/auth/LoginForm";
import { iniciarSesionDesdeFormulario } from "@/features/auth/login";
import { useAuthSession } from "@/features/auth/useAuthSession";
import type { LoginFormValues } from "@/types/schemas";

export default function LoginPage() {
  const { usuario } = useAuthSession();
  const router = useRouter();
  const [mensaje, setMensaje] = useState("");

  function enviar(
    datos: LoginFormValues,
    form: UseFormReturn<LoginFormValues>,
  ) {
    setMensaje("");
    return iniciarSesionDesdeFormulario(
      datos,
      form,
      setMensaje,
      () => {
        window.dispatchEvent(new Event("auth-session-changed"));
        router.push("/");
      },
    );
  }

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="auth" sesionActiva={Boolean(usuario)} />
        <main className="accounts-workspace">
          <LoginForm mensaje={mensaje} onSubmit={enviar} />
        </main>
      </div>
    </div>
  );
}
