"use client";

import Link from "next/link";
import { useState, type FormEvent } from "react";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { registrarUsuario } from "@/services/autenticacion";
import type { RegistrarUsuarioRequest } from "@/types/autenticacion";
import {
  parseErrorValidacion,
  type ErrorValidacionDTO,
} from "@/types/error_validacion";

export default function RegistroPage() {
  const [datos, setDatos] = useState<RegistrarUsuarioRequest>({
    nombre_usuario: "",
    nombre_visible: "",
    correo: "",
    contrasena: "",
  });
  const [error, setError] = useState<ErrorValidacionDTO | null>(null);
  const [mensaje, setMensaje] = useState("");
  const [registrado, setRegistrado] = useState(false);
  const [enviando, setEnviando] = useState(false);

  function actualizar(campo: keyof RegistrarUsuarioRequest, valor: string) {
    setDatos((actuales) => ({ ...actuales, [campo]: valor }));
  }

  async function enviar(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setMensaje("");
    setEnviando(true);

    try {
      await registrarUsuario(datos);
      setRegistrado(true);
    } catch (fallo: unknown) {
      const validacion = parseErrorValidacion(fallo);
      if (validacion) {
        setError(validacion);
      } else {
        setMensaje(
          typeof fallo === "string"
            ? fallo
            : "No se pudo completar el registro. Inténtalo de nuevo.",
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
                <form className="auth-form" onSubmit={enviar}>
                  <label htmlFor="nombre_usuario">Nombre de usuario</label>
                  <input
                    id="nombre_usuario"
                    name="nombre_usuario"
                    autoComplete="username"
                    minLength={3}
                    maxLength={30}
                    required
                    value={datos.nombre_usuario}
                    onChange={(event) =>
                      actualizar("nombre_usuario", event.target.value)
                    }
                    aria-invalid={error?.campo === "nombre_usuario"}
                  />
                  {error?.campo === "nombre_usuario" && (
                    <p className="auth-field-error">{error.mensaje}</p>
                  )}

                  <label htmlFor="nombre_visible">Nombre visible</label>
                  <input
                    id="nombre_visible"
                    name="nombre_visible"
                    autoComplete="name"
                    required
                    value={datos.nombre_visible}
                    onChange={(event) =>
                      actualizar("nombre_visible", event.target.value)
                    }
                    aria-invalid={error?.campo === "nombre_visible"}
                  />
                  {error?.campo === "nombre_visible" && (
                    <p className="auth-field-error">{error.mensaje}</p>
                  )}

                  <label htmlFor="correo">Correo</label>
                  <input
                    id="correo"
                    name="correo"
                    type="email"
                    autoComplete="email"
                    required
                    value={datos.correo}
                    onChange={(event) => actualizar("correo", event.target.value)}
                    aria-invalid={error?.campo === "correo"}
                  />
                  {error?.campo === "correo" && (
                    <p className="auth-field-error">{error.mensaje}</p>
                  )}

                  <label htmlFor="contrasena">Contraseña (mínimo 10 caracteres)</label>
                  <input
                    id="contrasena"
                    name="contrasena"
                    type="password"
                    autoComplete="new-password"
                    minLength={10}
                    required
                    value={datos.contrasena}
                    onChange={(event) =>
                      actualizar("contrasena", event.target.value)
                    }
                    aria-invalid={error?.campo === "contrasena"}
                  />
                  {error?.campo === "contrasena" && (
                    <p className="auth-field-error">{error.mensaje}</p>
                  )}

                  {error &&
                    ![
                      "nombre_usuario",
                      "nombre_visible",
                      "correo",
                      "contrasena",
                    ].includes(error.campo) && (
                      <p className="auth-error" role="alert">
                        {error.mensaje}
                      </p>
                    )}
                  {mensaje && (
                    <p className="auth-error" role="alert">
                      {mensaje}
                    </p>
                  )}
                  <button
                    className="account-primary"
                    type="submit"
                    disabled={enviando}
                  >
                    {enviando ? "Registrando…" : "Crear cuenta"}
                  </button>
                </form>
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
