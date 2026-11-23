"use client";

import { GroupExpenseForm, InviteMemberForm } from "@/components/social/GroupForms";
import type { Account, SyncStatus } from "@/types/accounts";
import type {
  DetalleGrupoDTO,
  ParteGastoDTO,
  RegistrarGastoRequestDTO,
  UsuarioResumenDTO,
} from "@/types/social";

export type GroupSyncView = {
  cuentas: Account[];
  seleccion: string;
  estado: SyncStatus;
  guardando: boolean;
  sincronizando: boolean;
  error: string;
  vincular: (cuentaId: string) => Promise<boolean>;
  sincronizar: () => Promise<boolean>;
};

type Props = {
  detalle: DetalleGrupoDTO;
  usuarioActual: string;
  amigos: UsuarioResumenDTO[];
  pending: boolean;
  sync: GroupSyncView;
  onInvite: (nombreUsuario: string) => Promise<boolean>;
  onRegisterExpense: (datos: RegistrarGastoRequestDTO) => Promise<boolean>;
  onPreview: (
    datos: RegistrarGastoRequestDTO,
  ) => Promise<ParteGastoDTO[]>;
};

function formatSyncTimestamp(timestamp: string) {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return timestamp;

  return new Intl.DateTimeFormat("es-VE", {
    dateStyle: "short",
    timeStyle: "short",
  }).format(date);
}

export function GroupDetail({
  detalle,
  usuarioActual,
  amigos,
  pending,
  sync,
  onInvite,
  onRegisterExpense,
  onPreview,
}: Props) {
  const miembroActual = detalle.miembros.find(
    (miembro) => miembro.nombre_usuario === usuarioActual,
  );
  const esAdministrador = miembroActual?.rol === "administrador";
  const nombresMiembros = detalle.miembros.map(
    (miembro) => miembro.nombre_usuario,
  );
  const cuentasElegibles = sync.cuentas.filter(
    (cuenta) => cuenta.activa && cuenta.moneda_codigo === detalle.moneda,
  );
  const cuentaVinculada = cuentasElegibles.find(
    (cuenta) => cuenta.id === sync.seleccion,
  );

  return (
    <div className="group-detail">
      <header className="group-detail-header">
        <div>
          <p className="group-eyebrow">{detalle.actividad ?? "Grupo compartido"}</p>
          <h2>{detalle.nombre}</h2>
          {detalle.descripcion && <p>{detalle.descripcion}</p>}
        </div>
        <span className="group-currency">{detalle.moneda}</span>
      </header>

      <section className="group-account-sync" aria-label="Cuenta y sincronización del grupo">
        <header className="group-account-sync-heading">
          <h3>Vincular cuenta</h3>
          {cuentaVinculada && (
            <div className="group-linked-account">
              <span>Cuenta vinculada</span>
              <strong>{cuentaVinculada.nombre}</strong>
            </div>
          )}
        </header>

        <div className="group-account-sync-controls">
          <select
            value={sync.seleccion}
            disabled={sync.guardando || cuentasElegibles.length === 0}
            onChange={(event) => void sync.vincular(event.target.value)}
          >
            <option value="">Selecciona una cuenta</option>
            {cuentasElegibles.map((cuenta) => (
              <option key={cuenta.id} value={cuenta.id}>
                {cuenta.nombre}
              </option>
            ))}
          </select>
          {cuentaVinculada && (
            <button
              className="account-primary"
              type="button"
              disabled={sync.sincronizando || sync.estado.sesion_vencida}
              onClick={() => void sync.sincronizar()}
            >
              {sync.sincronizando ? "Sincronizando..." : "Sincronizar"}
            </button>
          )}
        </div>

        <div className="group-account-sync-status">
          {sync.estado.ultima_sincronizacion && (
            <small>
              Última sincronización:{" "}
              {formatSyncTimestamp(sync.estado.ultima_sincronizacion)}
            </small>
          )}
          {sync.estado.sesion_vencida && (
            <small className="group-sync-error" role="alert">
              Tu sesión venció, inicia sesión de nuevo
            </small>
          )}
          {sync.error && (
            <small className="group-sync-error" role="alert">
              {sync.error}
            </small>
          )}
        </div>
      </section>

      <section className="group-panel" aria-labelledby="group-members-title">
        <div className="group-section-heading">
          <h3 id="group-members-title">Miembros</h3>
          <p>{detalle.miembros.length} miembros aceptados</p>
        </div>
        <div className="group-member-list">
          {detalle.miembros.map((miembro) => (
            <article
              className="group-member-card"
              key={miembro.nombre_usuario}
            >
              <span className="group-avatar" aria-hidden="true">
                {miembro.nombre_visible.slice(0, 1).toUpperCase()}
              </span>
              <span>
                <strong>{miembro.nombre_visible}</strong>
                <small>@{miembro.nombre_usuario}</small>
              </span>
              {miembro.rol === "administrador" && (
                <span className="group-role">Admin</span>
              )}
            </article>
          ))}
        </div>
        {esAdministrador && (
          <div className="group-invite-member">
            <InviteMemberForm
              amigos={amigos}
              excluidos={[usuarioActual, ...nombresMiembros]}
              pending={pending}
              onInvite={onInvite}
            />
          </div>
        )}
      </section>

      <section className="group-balances" aria-labelledby="group-balances-title">
        <div className="group-section-heading">
          <h3 id="group-balances-title">Balances</h3>
          <p>Positivo: le deben · negativo: debe</p>
        </div>
        <div className="group-balance-list">
          {Object.entries(detalle.balances).map(([nombre, balance]) => (
            <article className="group-balance-card" key={nombre}>
              <span>@{nombre}</span>
              <strong className={balance.startsWith("-") ? "is-negative" : "is-positive"}>
                {balance} {detalle.moneda}
              </strong>
            </article>
          ))}
        </div>
      </section>

      <GroupExpenseForm
        key={detalle.id}
        miembros={detalle.miembros}
        moneda={detalle.moneda}
        pending={pending}
        onSubmit={onRegisterExpense}
        onPreview={onPreview}
      />

      <section className="group-expenses" aria-labelledby="group-expenses-title">
        <div className="group-section-heading">
          <h3 id="group-expenses-title">Gastos compartidos</h3>
          <p>{detalle.gastos.length} gastos</p>
        </div>
        {detalle.gastos.length === 0 ? (
          <p className="group-empty">Todavía no hay gastos en este grupo.</p>
        ) : (
          <div className="group-expense-list">
            {[...detalle.gastos].reverse().map((gasto) => (
              <article className="group-expense-card" key={gasto.id}>
                <header>
                  <div>
                    <h4>{gasto.concepto}</h4>
                    <p>
                      {gasto.fecha} · Pagó @{gasto.pagador} ·{" "}
                      {gasto.regla === "iguales"
                        ? "Partes iguales"
                        : gasto.regla === "porcentajes"
                          ? "Porcentajes"
                          : "Montos fijos"}
                    </p>
                  </div>
                  <strong>
                    {gasto.monto} {detalle.moneda}
                  </strong>
                </header>
                <ul>
                  {gasto.partes.map((parte) => (
                    <li key={parte.usuario}>
                      <span>@{parte.usuario}</span>
                      <span>
                        {parte.monto} {detalle.moneda}
                        {parte.residuo && (
                          <small className="group-rounding">
                            Incluye 0,01 de redondeo
                          </small>
                        )}
                      </span>
                    </li>
                  ))}
                </ul>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
