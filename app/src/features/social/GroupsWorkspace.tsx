"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import type { UseFormReturn } from "react-hook-form";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { LoginForm } from "@/components/auth/LoginForm";
import { GroupDetail } from "@/components/social/GroupDetail";
import { CreateGroupForm } from "@/components/social/GroupForms";
import { iniciarSesionDesdeFormulario } from "@/features/auth/login";
import { useAuthSession } from "@/features/auth/useAuthSession";
import { useGroups } from "@/features/social/useGroups";
import { useSocial } from "@/features/social/useSocial";
import type {
  CrearGrupoRequestDTO,
  GrupoDTO,
  RegistrarGastoRequestDTO,
} from "@/types/social";
import type { LoginFormValues } from "@/types/schemas";

function GroupCard({
  grupo,
  seleccionado,
  onSelect,
}: {
  grupo: GrupoDTO;
  seleccionado: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      className={`group-list-card${seleccionado ? " is-selected" : ""}`}
      type="button"
      aria-pressed={seleccionado}
      onClick={onSelect}
    >
      <span className="group-list-icon" aria-hidden="true">
        ◎
      </span>
      <span>
        <strong>{grupo.nombre}</strong>
        <small>
          {grupo.actividad ?? "Grupo compartido"} · {grupo.moneda}
        </small>
      </span>
      <span aria-hidden="true">›</span>
    </button>
  );
}

export function GroupsWorkspace() {
  const {
    usuario,
    cargando: cargandoPerfil,
    esAplicacionTauri,
  } = useAuthSession();
  const grupos = useGroups(Boolean(usuario));
  const amistades = useSocial(Boolean(usuario));
  const router = useRouter();
  const [mensajeLogin, setMensajeLogin] = useState("");
  const gruposActivos = grupos.grupos.miembros;
  const grupoSeleccionado = grupos.grupoSeleccionado;
  const cargarDetalle = grupos.cargarDetalle;

  useEffect(() => {
    if (!grupoSeleccionado && gruposActivos.length > 0) {
      void cargarDetalle(gruposActivos[0].id);
    }
  }, [cargarDetalle, grupoSeleccionado, gruposActivos]);

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
        router.push("/grupos/");
      },
    );
  }

  async function crearGrupo(datos: CrearGrupoRequestDTO) {
    return grupos.crear(datos);
  }

  async function registrarGasto(datos: RegistrarGastoRequestDTO) {
    if (!grupos.grupoSeleccionado) return false;
    return grupos.registrarGasto(grupos.grupoSeleccionado, datos);
  }

  const invitaciones = grupos.grupos.invitaciones;

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar
          active="groups"
          sesionActiva={Boolean(usuario)}
        />
        <main className="accounts-workspace">
          {!esAplicacionTauri ? (
            <section className="group-access-message" role="status">
              <h1>Grupos</h1>
              <p>
                Las funciones sociales requieren la aplicación de escritorio y
                conexión al servidor.
              </p>
            </section>
          ) : cargandoPerfil ? (
            <p className="profile-loading" role="status">
              Consultando tu sesión…
            </p>
          ) : !usuario ? (
            <section className="group-login">
              <div className="group-page-heading">
                <p className="breadcrumb">
                  Inicio <span>/</span> Grupos
                </p>
                <h1>Grupos de amigos</h1>
                <p>Inicia sesión para dividir gastos compartidos.</p>
              </div>
              <LoginForm mensaje={mensajeLogin} onSubmit={enviarLogin} />
            </section>
          ) : (
            <>
              <header className="accounts-header group-page-header">
                <div>
                  <p className="breadcrumb">
                    Inicio <span>/</span> Grupos
                  </p>
                  <h1>Grupos de amigos</h1>
                  <p>
                    Hola, {usuario.nombre_visible}. Organiza gastos compartidos
                    y mantén los balances claros.
                  </p>
                </div>
                <button
                  className="account-secondary"
                  type="button"
                  onClick={() => void grupos.cargarGrupos()}
                  disabled={grupos.cargando}
                >
                  {grupos.cargando ? "Actualizando…" : "Actualizar"}
                </button>
              </header>

              {(grupos.error || amistades.error) && (
                <p className="group-page-error" role="alert">
                  {grupos.error || amistades.error}
                </p>
              )}

              <div className="groups-layout">
                <aside className="groups-navigation">
                  <CreateGroupForm
                    pending={grupos.guardando}
                    onCreate={crearGrupo}
                  />

                  <section
                    className="group-panel"
                    aria-labelledby="group-invitations-title"
                  >
                    <div className="group-section-heading">
                      <h2 id="group-invitations-title">Invitaciones</h2>
                    </div>
                    {invitaciones.length === 0 ? (
                      <p className="group-muted">
                        No tienes invitaciones de grupo pendientes.
                      </p>
                    ) : (
                      <div className="group-invitation-list">
                        {invitaciones.map((invitacion) => (
                          <article
                            className="group-invitation-card"
                            key={invitacion.id}
                          >
                            <strong>{invitacion.nombre}</strong>
                            <small>
                              {invitacion.actividad ?? "Grupo compartido"} ·{" "}
                              {invitacion.moneda}
                            </small>
                            <div>
                              <button
                                className="account-primary"
                                type="button"
                                disabled={grupos.guardando}
                                onClick={() =>
                                  void grupos.responderInvitacion(
                                    invitacion.id,
                                    true,
                                  )
                                }
                              >
                                Aceptar
                              </button>
                              <button
                                className="account-secondary"
                                type="button"
                                disabled={grupos.guardando}
                                onClick={() =>
                                  void grupos.responderInvitacion(
                                    invitacion.id,
                                    false,
                                  )
                                }
                              >
                                Rechazar
                              </button>
                            </div>
                          </article>
                        ))}
                      </div>
                    )}
                  </section>

                  <section
                    className="group-panel"
                    aria-labelledby="my-groups-title"
                  >
                    <div className="group-section-heading">
                      <h2 id="my-groups-title">Mis grupos</h2>
                      <p>{gruposActivos.length}</p>
                    </div>
                    {gruposActivos.length === 0 ? (
                      <p className="group-muted">
                        Aún no perteneces a ningún grupo.
                      </p>
                    ) : (
                      <div className="group-list">
                        {gruposActivos.map((grupo) => (
                          <GroupCard
                            key={grupo.id}
                            grupo={grupo}
                            seleccionado={grupo.id === grupoSeleccionado}
                            onSelect={() => void cargarDetalle(grupo.id)}
                          />
                        ))}
                      </div>
                    )}
                  </section>
                </aside>

                <section className="groups-content">
                  {grupos.cargandoDetalle ? (
                    <p className="group-empty" role="status">
                      Cargando grupo…
                    </p>
                  ) : grupos.detalle ? (
                    <GroupDetail
                      detalle={grupos.detalle}
                      usuarioActual={usuario.nombre_usuario}
                      amigos={amistades.amistades.amigos}
                      pending={grupos.guardando}
                      onPreview={grupos.previsualizar}
                      onInvite={(nombreUsuario) =>
                        grupos.invitar(
                          grupos.detalle!.id,
                          nombreUsuario,
                        )
                      }
                      onRegisterExpense={registrarGasto}
                    />
                  ) : (
                    <div className="group-empty">
                      <span aria-hidden="true">◎</span>
                      <h2>Selecciona un grupo</h2>
                      <p>
                        Elige un grupo para ver sus miembros, gastos y balances.
                      </p>
                    </div>
                  )}
                </section>
              </div>
            </>
          )}
        </main>
      </div>
    </div>
  );
}
