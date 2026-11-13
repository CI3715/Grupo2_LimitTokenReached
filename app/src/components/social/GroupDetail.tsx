"use client";

import { GroupExpenseForm, InviteMemberForm } from "@/components/social/GroupForms";
import type {
  DetalleGrupoDTO,
  ParteGastoDTO,
  RegistrarGastoRequestDTO,
  UsuarioResumenDTO,
} from "@/types/social";

type Props = {
  detalle: DetalleGrupoDTO;
  usuarioActual: string;
  amigos: UsuarioResumenDTO[];
  pending: boolean;
  onInvite: (nombreUsuario: string) => Promise<boolean>;
  onRegisterExpense: (datos: RegistrarGastoRequestDTO) => Promise<boolean>;
  onPreview: (
    datos: RegistrarGastoRequestDTO,
  ) => Promise<ParteGastoDTO[]>;
};

export function GroupDetail({
  detalle,
  usuarioActual,
  amigos,
  pending,
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
