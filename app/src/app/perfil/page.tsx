"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import type { FormEvent } from "react";
import type { UseFormReturn } from "react-hook-form";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { LoginForm } from "@/components/auth/LoginForm";
import { iniciarSesionDesdeFormulario } from "@/features/auth/login";
import { useSocial } from "@/features/social/useSocial";
import { useAuthSession } from "@/features/auth/useAuthSession";
import type { LoginFormValues } from "@/types/schemas";
import type { InvitacionAmistadDTO } from "@/types/social";
import "./perfil.css";

export default function PerfilPage() {
  const { usuario, cargando, salir } = useAuthSession();
  const {
    amistades,
    usuarioEncontrado,
    cargando: cargandoAmistades,
    error: errorSocial,
    buscar,
    invitar,
    responder,
    eliminar,
  } = useSocial(Boolean(usuario));
  const router = useRouter();
  const [mostrarNotificaciones, setMostrarNotificaciones] = useState(false);
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

  async function buscarAmigo(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!busquedaAmigo.trim()) return;
    await buscar(busquedaAmigo);
  }

  async function responderInvitacion(
    invitacion: InvitacionAmistadDTO,
    aceptar: boolean,
  ) {
    await responder(invitacion, aceptar);
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
                    <div className="friend-search-area">
                      <form
                        className="friend-search-bar"
                        onSubmit={(event) => void buscarAmigo(event)}
                      >
                        <input
                          id="search-friend-username"
                          value={busquedaAmigo}
                          onChange={(event) =>
                            setBusquedaAmigo(event.target.value)
                          }
                          placeholder="Nombre de usuario"
                        />
                        <button
                          className="account-primary"
                          type="submit"
                          disabled={cargandoAmistades}
                        >
                          Buscar
                        </button>
                      </form>
                      {usuarioEncontrado && (
                        <div className="friend-search-result">
                          <div>
                            <strong>{usuarioEncontrado.nombre_visible}</strong>
                            <span>{usuarioEncontrado.nombre_usuario}</span>
                          </div>
                          <button
                            className="account-primary"
                            type="button"
                            onClick={async () => {
                              const enviada = await invitar(
                                usuarioEncontrado.nombre_usuario,
                              );
                              if (enviada) {
                                setBusquedaAmigo("");
                              }
                            }}
                          >
                            Agregar
                          </button>
                        </div>
                      )}
                    </div>
                    <div
                      className="notifications-wrapper"
                      ref={notificationsRef}
                    >
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
                          {amistades.recibidas.length === 0 ? (
                            "No tienes invitaciones pendientes."
                          ) : (
                            <div className="notification-list">
                              {amistades.recibidas.map((invitacion) => (
                                <div
                                  className="notification-item"
                                  key={invitacion.id}
                                >
                                  <span>
                                    {invitacion.nombre_visible} (
                                    {invitacion.nombre_usuario}) te invitó a
                                    ser su amigo.
                                  </span>
                                  <div>
                                    <button
                                      className="account-primary"
                                      type="button"
                                      onClick={() =>
                                        void responderInvitacion(
                                          invitacion,
                                          true,
                                        )
                                      }
                                    >
                                      Aceptar
                                    </button>
                                    <button
                                      className="account-secondary"
                                      type="button"
                                      onClick={() =>
                                        void responderInvitacion(
                                          invitacion,
                                          false,
                                        )
                                      }
                                    >
                                      Rechazar
                                    </button>
                                  </div>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                  {amistades.enviadas.length > 0 && (
                    <section
                      className="friends-section"
                      aria-labelledby="outgoing-friend-invitations"
                    >
                      <div className="section-heading">
                        <h2 id="outgoing-friend-invitations">
                          Invitaciones enviadas
                        </h2>
                      </div>
                      <div className="friends-list">
                        {amistades.enviadas.map((invitacion) => (
                          <article className="friend-card" key={invitacion.id}>
                            <div>
                              <strong>{invitacion.nombre_visible}</strong>
                              <span>{invitacion.nombre_usuario}</span>
                            </div>
                            <span>Esperando respuesta</span>
                          </article>
                        ))}
                      </div>
                    </section>
                  )}
                  <section className="friends-section" aria-labelledby="friends-title">
                    <div className="section-heading">
                      <h2 id="friends-title">Amigos</h2>
                    </div>
                    {errorSocial && (
                      <p className="profile-social-error" role="alert">
                        {errorSocial}
                      </p>
                    )}
                    {cargandoAmistades ? (
                      <p className="profile-loading" role="status">
                        Cargando amistades…
                      </p>
                    ) : amistades.amigos.length === 0 ? (
                      <div className="friends-empty">
                        <p>Aún no tienes amigos.</p>
                        <span>Cuando agregues amigos, aparecerán aquí.</span>
                      </div>
                    ) : (
                      <div className="friends-list">
                        {amistades.amigos.map((amigo) => (
                          <article
                            className="friend-card"
                            key={amigo.nombre_usuario}
                          >
                            <div>
                              <strong>{amigo.nombre_visible}</strong>
                              <span>{amigo.nombre_usuario}</span>
                            </div>
                            <button
                              className="account-secondary"
                              type="button"
                              onClick={() => void eliminar(amigo.nombre_usuario)}
                            >
                              Eliminar
                            </button>
                          </article>
                        ))}
                      </div>
                    )}
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
