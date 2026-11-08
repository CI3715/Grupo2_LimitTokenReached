"use client";

import Link from "next/link";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { useAuthSession } from "@/features/auth/useAuthSession";
import "../accounts.css";

export default function PerfilPage() {
  const {
    usuario,
    cargando,
    errorPerfil,
    esAplicacionTauri,
    cargarPerfil,
  } = useAuthSession();
  const fechaAlta = usuario ? new Date(usuario.fecha_alta) : null;
  const fechaAltaFormateada =
    fechaAlta && !Number.isNaN(fechaAlta.getTime())
      ? new Intl.DateTimeFormat("es-VE", {
          dateStyle: "medium",
          timeStyle: "short",
        }).format(fechaAlta)
      : (usuario?.fecha_alta ?? "");

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="profile" />
        <main className="accounts-workspace">
          <header className="accounts-header tasas-header">
            <div>
              <p className="breadcrumb">
                Inicio <span>/</span> Mi perfil
              </p>
              <h1>Mi perfil</h1>
              
            </div>
          </header>

          <section className="auth-card profile-card" aria-labelledby="profile-title">
            <h2 id="profile-title">Datos del usuario</h2>
            {cargando ? (
              <p role="status">Consultando tu perfil…</p>
            ) : usuario ? (
              <>
                <dl className="profile-details">
                  <div>
                    <dt>Nombre visible</dt>
                    <dd>{usuario.nombre_visible}</dd>
                  </div>
                  <div>
                    <dt>Nombre de usuario</dt>
                    <dd>{usuario.nombre_usuario}</dd>
                  </div>
                  <div>
                    <dt>Correo</dt>
                    <dd>{usuario.correo}</dd>
                  </div>
                  <div>
                    <dt>Fecha de registro</dt>
                    <dd>{fechaAltaFormateada}</dd>
                  </div>
                </dl>
                <button
                  className="account-primary"
                  type="button"
                  onClick={cargarPerfil}
                  disabled={cargando}
                >
                  Actualizar perfil
                </button>
              </>
            ) : (
              <div className="profile-empty">
                <p className="auth-error" role="alert">
                  {errorPerfil || "No hay una sesión iniciada."}
                </p>
                {esAplicacionTauri && (
                  <Link className="auth-link-button" href="/login/">
                    Iniciar sesión
                  </Link>
                )}
                {esAplicacionTauri && (
                  <button
                    className="account-primary"
                    type="button"
                    onClick={cargarPerfil}
                    disabled={cargando}
                  >
                    Reintentar consulta
                  </button>
                )}
              </div>
            )}
          </section>
        </main>
      </div>
    </div>
  );
}
