"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState, type FormEvent } from "react";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { iniciarSesion } from "@/services/autenticacion";
import type { IniciarSesionRequest } from "@/types/autenticacion";
import {
  parseErrorValidacion,
  type ErrorValidacionDTO,
} from "@/types/error_validacion";

export default function LoginPage() {
  const router = useRouter();
  const [correo, setCorreo] = useState("");
  const [contrasena, setContrasena] = useState("");
  const [error, setError] = useState<ErrorValidacionDTO | null>(null);
  const [mensaje, setMensaje] = useState("");
  const [enviando, setEnviando] = useState(false);

  async function enviar(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setMensaje("");
    setEnviando(true);

    const datos: IniciarSesionRequest = { correo, contrasena };
    try {
      await iniciarSesion(datos);
      window.dispatchEvent(new Event("auth-session-changed"));
      router.replace("/");
    } catch (fallo: unknown) {
      const validacion = parseErrorValidacion(fallo);
      if (validacion) {
        setError(validacion);
      } else {
        setMensaje(
          typeof fallo === "string"
            ? fallo
            : "No se pudo iniciar sesión. Inténtalo de nuevo.",
        );
      }
    } finally {
      setEnviando(false);
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
            <form className="auth-form" onSubmit={enviar}>
              <label htmlFor="correo">Correo</label>
              <input
                id="correo"
                name="correo"
                type="email"
                autoComplete="email"
                required
                value={correo}
                onChange={(event) => setCorreo(event.target.value)}
                aria-invalid={error?.campo === "correo"}
              />
              {error?.campo === "correo" && (
                <p className="auth-field-error">{error.mensaje}</p>
              )}

              <label htmlFor="contrasena">Contraseña</label>
              <input
                id="contrasena"
                name="contrasena"
                type="password"
                autoComplete="current-password"
                required
                value={contrasena}
                onChange={(event) => setContrasena(event.target.value)}
                aria-invalid={error?.campo === "contrasena"}
              />
              {error?.campo === "contrasena" && (
                <p className="auth-field-error">{error.mensaje}</p>
              )}

              {error?.campo === "credenciales" && (
                <p className="auth-error" role="alert">
                  {error.mensaje}
                </p>
              )}
              {error &&
                !["correo", "contrasena", "credenciales"].includes(
                  error.campo,
                ) && (
                  <p className="auth-error" role="alert">
                    {error.mensaje}
                  </p>
                )}
              {mensaje && (
                <p className="auth-error" role="alert">
                  {mensaje}
                </p>
              )}

              <button className="account-primary" type="submit" disabled={enviando}>
                {enviando ? "Conectando…" : "Iniciar sesión"}
              </button>
            </form>
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
