"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import type { UseFormReturn } from "react-hook-form";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { LoginForm } from "@/components/auth/LoginForm";
import { iniciarSesionDesdeFormulario } from "@/features/auth/login";
import { useAuthSession } from "@/features/auth/useAuthSession";
import type { LoginFormValues } from "@/types/schemas";
import "./perfil.css";

type FriendPanel = "add" | "search" | null;

export default function PerfilPage() {
  const { usuario, cargando, salir } = useAuthSession();
  const router = useRouter();
  const [friendPanel, setFriendPanel] = useState<FriendPanel>(null);
  const [mostrarNotificaciones, setMostrarNotificaciones] = useState(false);
  const [nombreUsuarioAmigo, setNombreUsuarioAmigo] = useState("");
  const [busquedaAmigo, setBusquedaAmigo] = useState("");
  const [mensajeLogin, setMensajeLogin] = useState("");
  const notificationsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function cerrarNotificaciones(event: MouseEvent) {
      if (
        notificationsRef.current &&
        !notificationsRef.current.contains(event.target as Node)
      ) {
        setMostrarNotificaciones(false);
      }
    }

    document.addEventListener("mousedown", cerrarNotificaciones);
    return () => document.removeEventListener("mousedown", cerrarNotificaciones);
  }, []);

  function enviarLogin(
    datos: LoginFormValues,
    form: UseFormReturn<LoginFormValues>,
  ) {
    setMensajeLogin("");
    return iniciarSesionDesdeFormulario(
      datos,
      form,
      setMensajeLogin,
      () => {
        window.dispatchEvent(new Event("auth-session-changed"));
        router.push("/");
      },
    );
  }

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar
          active={usuario ? "profile" : "auth"}
          sesionActiva={Boolean(usuario)}
        />
        <main className="accounts-workspace">
          {!usuario ? (
            <LoginForm mensaje={mensajeLogin} onSubmit={enviarLogin} />
          ) : (
            <>
              <header className="accounts-header tasas-header">
                <div>
                  <p className="breadcrumb">
                    Inicio <span>/</span> Mi perfil
                  </p>
                  <h1>{usuario.nombre_visible}</h1>
                  <p className="profile-heading-subtitle">
                    {usuario.nombre_usuario}
                  </p>
                </div>
                <button
                  className="account-primary"
                  type="button"
                  onClick={() => void salir()}
                >
                  Cerrar sesión
                </button>
              </header>

              {cargando ? (
                <p className="profile-loading" role="status">
                  Consultando tu perfil…
                </p>
              ) : (
                <>
                  <div className="profile-floating-actions">
                    <div className="friend-action-group">
                      <button
                        className="account-secondary friends-action"
                        type="button"
                        aria-expanded={friendPanel === "add"}
                        onClick={() =>
                          setFriendPanel((current) =>
                            current === "add" ? null : "add",
                          )
                        }
                      >
                        <span aria-hidden="true">＋</span> Agregar amigo
                      </button>
                      <form
                        className={`friend-search-bar ${
                          friendPanel === "add" ? "is-open" : ""
                        }`}
                        onSubmit={(event) => event.preventDefault()}
                      >
                        <input
                          id="add-friend-username"
                          value={nombreUsuarioAmigo}
                          onChange={(event) =>
                            setNombreUsuarioAmigo(event.target.value)
                          }
                          placeholder="Escribe un nombre de usuario"
                        />
                        <button className="account-primary" type="submit">
                          Agregar
                        </button>
                      </form>
                    </div>
                    <div className="friend-action-group">
                      <button
                        className="account-secondary friends-action"
                        type="button"
                        aria-expanded={friendPanel === "search"}
                        onClick={() =>
                          setFriendPanel((current) =>
                            current === "search" ? null : "search",
                          )
                        }
                      >
                        <span aria-hidden="true">🔍</span> Buscar amigo
                      </button>
                      <div
                        className={`friend-search-bar ${
                          friendPanel === "search" ? "is-open" : ""
                        }`}
                      >
                        <input
                          id="search-friend-username"
                          value={busquedaAmigo}
                          onChange={(event) =>
                            setBusquedaAmigo(event.target.value)
                          }
                          placeholder="Escribe un nombre de usuario"
                        />
                      </div>
                    </div>
                    <div className="notifications-wrapper" ref={notificationsRef}>
                      <button
                        className="account-secondary notification-action"
                        type="button"
                        aria-label="Abrir notificaciones"
                        aria-expanded={mostrarNotificaciones}
                        onClick={() =>
                          setMostrarNotificaciones((visible) => !visible)
                        }
                      >
                        <span aria-hidden="true">🔔</span>
                      </button>
                      {mostrarNotificaciones && (
                        <div className="notifications-menu" role="status">
                          No tienes invitaciones pendientes.
                        </div>
                      )}
                    </div>
                  </div>
                  <section className="friends-section" aria-labelledby="friends-title">
                    <div className="section-heading">
                      <h2 id="friends-title">Amigos</h2>
                    </div>
                    <div className="friends-empty">
                      <p>Aún no tienes amigos.</p>
                      <span>Cuando agregues amigos, aparecerán aquí.</span>
                    </div>
                  </section>
                </>
              )}
            </>
          )}
        </main>
      </div>
    </div>
  );
}
