"use client";

import Link from "next/link";
import { formatMoney, minorUnits } from "@/types/account-money";
import type { Account, Currency } from "@/types/accounts";
import type { MovimientoDTO } from "@/types/movements";
import { movementCategories } from "@/types/movement-categories";
import { MovementForm } from "@/components/movements/movement-form";
import type { RegistrarMovimientoRequest, MovimientoDTO as MovementResponse } from "@/types/movements";

function amount(value: bigint, currency?: Currency) {
  return `${currency?.simbolo ?? ""} ${formatMoney(value)}`.trim();
}

function movementEffect(movement: MovimientoDTO): bigint {
  const value = minorUnits(movement.monto);
  return movement.tipo === "Gasto" ? -value : value;
}

export function AccountDetail({
  account,
  currency,
  movements,
  movementsError,
  formOpen,
  onOpenMovementForm,
  onCloseMovementForm,
  onMovementRegistered,
  onRegister,
}: {
  account: Account;
  currency?: Currency;
  movements: MovimientoDTO[];
  movementsError: string | null;
  formOpen: boolean;
  onOpenMovementForm: () => void;
  onCloseMovementForm: () => void;
  onMovementRegistered: (movement: MovimientoDTO) => void | Promise<void>;
  onRegister: (request: RegistrarMovimientoRequest) => Promise<MovementResponse>;
}) {
  const currentBalance = minorUnits(account.saldo);
  const income = movements
    .filter((movement) => movement.tipo === "Ingreso" && !movement.es_correccion)
    .reduce((total, movement) => total + minorUnits(movement.monto), BigInt(0));
  const expenses = movements
    .filter((movement) => movement.tipo === "Gasto" && !movement.es_correccion)
    .reduce((total, movement) => total + minorUnits(movement.monto), BigInt(0));
  const adjustments = movements
    .filter((movement) => movement.es_correccion)
    .reduce((total, movement) => total + movementEffect(movement), BigInt(0));
  const initial = currentBalance - income + expenses - adjustments;

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <aside className="accounts-sidebar">
          <Link href="/" className="accounts-brand">
            <span className="brand-mark" aria-hidden="true">C</span>
            <span>Cuentas Claras<small>finanzas locales</small></span>
          </Link>
          <p className="eyebrow">Principal</p>
          <nav aria-label="Principal">
            <Link href="/"><span aria-hidden="true">▦</span> Mis cuentas</Link>
            <button disabled><span aria-hidden="true">↕</span> Movimientos <small>Pronto</small></button>
            <button disabled><span aria-hidden="true">◎</span> Grupos <small>Pronto</small></button>
          </nav>
          <div className="sidebar-divider" />
          <p className="eyebrow">Sistema</p>
          <nav aria-label="Sistema">
            <button disabled><span aria-hidden="true">⚙</span> Preferencias <small>Pronto</small></button>
            <Link href="/conexion/"><span aria-hidden="true">⌁</span> Conexión</Link>
          </nav>
        </aside>
        <main className="accounts-workspace account-detail-workspace">
          <header className="accounts-header">
            <div>
              <p className="breadcrumb"><Link href="/">Mis cuentas</Link> <span>/</span> {account.nombre}</p>
              <h1>{account.nombre}</h1>
              <p>{account.tipo.replace("_", " ")} · {account.moneda_codigo}</p>
            </div>
            <div className="account-detail-actions">
              <button type="button" className="account-primary" onClick={onOpenMovementForm}>
                <span aria-hidden="true">＋</span> Registrar movimiento
              </button>
              <Link href="/" className="account-secondary">← Volver a cuentas</Link>
            </div>
          </header>

          {formOpen && currency && (
            <div className="movement-form-overlay" role="dialog" aria-modal="true" aria-labelledby="movement-dialog-title">
              <div className="movement-form-dialog">
                <div id="movement-dialog-title" className="movement-dialog-heading">
                  <h2>Nuevo movimiento</h2>
                  <button type="button" className="account-secondary" onClick={onCloseMovementForm}>Cerrar</button>
                </div>
                <MovementForm
                  account={account}
                  currency={currency}
                  categories={movementCategories}
                  onRegister={onRegister}
                  onCancel={onCloseMovementForm}
                  onRegistered={onMovementRegistered}
                />
              </div>
            </div>
          )}

          <section className="account-balance-hero" aria-labelledby="current-balance-title">
            <div>
              <p id="current-balance-title">Saldo actual</p>
              <strong>{amount(currentBalance, currency)}</strong>
            </div>
            <span className="currency-chip">{account.moneda_codigo}</span>
          </section>

          <section className="balance-composition" aria-labelledby="composition-title">
            <div className="section-heading">
              <div>
                <h2 id="composition-title">Composición del saldo</h2>
                <p>Cómo se forma el saldo actual de tu cuenta</p>
              </div>
            </div>
            <div className="composition-grid">
              <div><span className="composition-dot initial" /><span>Saldo inicial</span><strong>{amount(initial, currency)}</strong></div>
              <div><span className="composition-dot income" /><span>Ingresos</span><strong>{amount(income, currency)}</strong></div>
              <div><span className="composition-dot expense" /><span>Gastos</span><strong>{amount(-expenses, currency)}</strong></div>
              <div><span className="composition-dot adjustment" /><span>Ajustes</span><strong>{amount(adjustments, currency)}</strong></div>
            </div>
          </section>

          <section className="movements-list" aria-labelledby="movements-title">
            <div className="section-heading">
              <div>
                <h2 id="movements-title">Movimientos</h2>
                <p>{movements.length} {movements.length === 1 ? "movimiento" : "movimientos"}</p>
              </div>
            </div>
            {movementsError && <p className="account-error" role="alert">{movementsError}</p>}
            {movements.length ? (
              <ul>
                {movements.map((movement) => {
                  const isExpense = movement.tipo === "Gasto";
                  const value = minorUnits(movement.monto);
                  const signedValue = isExpense ? -value : value;
                  return (
                    <li className="movement-row" key={movement.id}>
                      <span className={`movement-icon ${isExpense ? "expense" : movement.es_correccion ? "adjustment" : "income"}`} aria-hidden="true">
                        {isExpense ? "−" : movement.es_correccion ? "↕" : "+"}
                      </span>
                      <div className="movement-description">
                        <strong>{movement.es_correccion ? "Ajuste" : movement.tipo}</strong>
                        <span>{movement.nota || "Sin nota"} · {movement.fecha}</span>
                      </div>
                      <strong className={`movement-amount ${isExpense ? "expense" : "income"}`}>
                        {amount(signedValue, currency)}
                      </strong>
                    </li>
                  );
                })}
              </ul>
            ) : (
              <p className="empty-movements">Esta cuenta todavía no tiene movimientos.</p>
            )}
          </section>
        </main>
      </div>
    </div>
  );
}
