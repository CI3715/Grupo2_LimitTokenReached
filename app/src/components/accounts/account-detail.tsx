"use client";

import Link from "next/link";
import { useState } from "react";
import { formatMoney, minorUnits } from "@/types/account-money";
import type { Account, Currency } from "@/types/accounts";
import type { MovimientoDTO } from "@/types/movements";
import { MovementForm } from "@/components/movements/movement-form";
import type { CategoriaDTO } from "@/types/movements";
import type { CrearCategoriaRequest } from "@/types/categories";
import type {
  CorregirMovimientoRequest,
  RegistrarMovimientoRequest,
  MovimientoDTO as MovementResponse,
} from "@/types/movements";

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
  onCorrect,
  categories,
  categoriesLoading,
  categoriesError,
  onRetryCategories,
  onCreateCategory,
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
  onCorrect: (request: CorregirMovimientoRequest) => Promise<MovementResponse>;
  categories: CategoriaDTO[];
  categoriesLoading: boolean;
  categoriesError: string | null;
  onRetryCategories: () => void;
  onCreateCategory: (request: CrearCategoriaRequest) => Promise<CategoriaDTO>;
}) {
  const [correctionMovement, setCorrectionMovement] = useState<MovimientoDTO | null>(null);
  const [correctionAmount, setCorrectionAmount] = useState("");
  const [correctionPending, setCorrectionPending] = useState(false);
  const [correctionError, setCorrectionError] = useState<string | null>(null);
  const [correctionNeedsConfirmation, setCorrectionNeedsConfirmation] = useState(false);
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
  const correctionByOriginal = new Map<string, MovimientoDTO[]>();
  for (const movement of movements) {
    if (!movement.es_correccion || !movement.movimiento_original_id) continue;
    const corrections = correctionByOriginal.get(movement.movimiento_original_id) ?? [];
    corrections.push(movement);
    correctionByOriginal.set(movement.movimiento_original_id, corrections);
  }

  async function submitCorrection(confirmNegativeBalance = false) {
    if (!correctionMovement || correctionPending) return;
    const amount = correctionAmount.trim().replace(",", ".");
    if (!/^\d+(?:\.\d{1,2})?$/.test(amount)) {
      setCorrectionError("Ingresa un monto válido con hasta dos decimales.");
      return;
    }

    setCorrectionPending(true);
    setCorrectionError(null);
    try {
      await onCorrect({
        movimiento_id: correctionMovement.id,
        monto_correcto: amount,
        confirmar_saldo_negativo: confirmNegativeBalance,
      });
      await onMovementRegistered(correctionMovement);
      setCorrectionMovement(null);
      setCorrectionNeedsConfirmation(false);
    } catch (error: unknown) {
      const failure = error as { campo?: unknown; mensaje?: unknown };
      const needsConfirmation = failure.campo === "saldo";
      setCorrectionNeedsConfirmation(needsConfirmation);
      setCorrectionError(
        typeof failure.mensaje === "string"
          ? failure.mensaje
          : "No se pudo corregir el movimiento.",
      );
    } finally {
      setCorrectionPending(false);
    }
  }

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
                </div>
                <MovementForm
                  account={account}
                  currency={currency}
                  categories={categories}
                  categoriesLoading={categoriesLoading}
                  categoriesError={categoriesError ?? undefined}
                  onRetryCategories={onRetryCategories}
                  onCreateCategory={onCreateCategory}
                  onRegister={onRegister}
                  onCancel={onCloseMovementForm}
                  onRegistered={onMovementRegistered}
                />
              </div>
            </div>
          )}
          {correctionMovement && currency && (
            <div className="movement-form-overlay" role="dialog" aria-modal="true" aria-labelledby="correction-dialog-title">
              <div className="movement-form-dialog correction-dialog">
                <div className="movement-dialog-heading">
                  <div>
                    <h2 id="correction-dialog-title">Corregir movimiento</h2>
                    <p>Monto registrado: {amount(minorUnits(correctionMovement.monto), currency)}</p>
                  </div>
                </div>
                <div className="correction-dialog-body">
                  <label htmlFor="corrected-amount">Monto correcto ({account.moneda_codigo})</label>
                  <input
                    id="corrected-amount"
                    type="text"
                    inputMode="decimal"
                    value={correctionAmount}
                    onChange={(event) => {
                      setCorrectionAmount(event.target.value);
                      setCorrectionNeedsConfirmation(false);
                      setCorrectionError(null);
                    }}
                    disabled={correctionPending}
                    autoFocus
                  />
                </div>
                {correctionError && <p className="account-error" role="alert">{correctionError}</p>}
                <div className="correction-actions">
                  <button type="button" className="account-secondary" onClick={() => setCorrectionMovement(null)} disabled={correctionPending}>
                    Cancelar
                  </button>
                  <button type="button" className="account-primary" onClick={() => void submitCorrection(correctionNeedsConfirmation)} disabled={correctionPending}>
                    {correctionPending
                      ? "Corrigiendo..."
                      : correctionNeedsConfirmation
                        ? "Confirmar y guardar"
                        : "Guardar corrección"}
                  </button>
                </div>
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
                  if (movement.es_correccion) return null;
                  const isExpense = movement.tipo === "Gasto";
                  const value = minorUnits(movement.monto);
                  const signedValue = isExpense ? -value : value;
                  const corrections = correctionByOriginal.get(movement.id) ?? [];
                  const isCorrected = corrections.length > 0;
                  return (
                    <li key={movement.id}>
                      <div className={`movement-row ${isCorrected ? "corrected" : ""}`}>
                        <span className={`movement-icon ${isExpense ? "expense" : movement.es_correccion ? "adjustment" : "income"}`} aria-hidden="true">
                          {isExpense ? "−" : movement.es_correccion ? "↕" : "+"}
                        </span>
                        <div className="movement-description">
                          <strong>{movement.es_correccion ? "Ajuste de corrección" : movement.tipo}</strong>
                          <span>{movement.nota || "Sin nota"} · {movement.fecha}</span>
                          {!movement.es_correccion && isCorrected && <small className="movement-corrected-label">Corregido</small>}
                        </div>
                        <strong className={`movement-amount ${isExpense ? "expense" : "income"}`}>
                          {amount(signedValue, currency)}
                        </strong>
                        {!movement.es_correccion && (
                          <button
                            type="button"
                            className="account-secondary movement-correct-button"
                            onClick={() => {
                              setCorrectionMovement(movement);
                              setCorrectionAmount(movement.monto);
                              setCorrectionError(null);
                            }}
                          >
                            {isCorrected ? "Corregir nuevamente" : "Corregir"}
                          </button>
                        )}
                      </div>
                      {corrections.map((correction) => (
                        <div className="movement-row movement-adjustment-row" key={correction.id}>
                          <span className="movement-icon adjustment" aria-hidden="true">↕</span>
                          <div className="movement-description">
                            <strong>Ajuste de corrección</strong>
                            <span>Nuevo monto: {amount(value + minorUnits(correction.monto), currency)} · {correction.fecha}</span>
                          </div>
                          <strong className={`movement-amount ${isExpense ? "expense" : "income"}`}>
                            {amount(movementEffect(correction), currency)}
                          </strong>
                        </div>
                      ))}
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
