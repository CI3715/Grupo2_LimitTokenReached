"use client";

import Decimal from "decimal.js";
import { useCallback, useEffect, useMemo, useState } from "react";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { RegistrarTasaForm } from "@/components/patrimonio/RegistrarTasaForm";
import { deleteRate } from "@/services/deleteRate";
import { listCurrencies } from "@/services/listCurrencies";
import { listRateHistory } from "@/services/listRateHistory";
import { registrarTasa } from "@/services/registrarTasa";
import type { Currency } from "@/types/accounts";
import { parseErrorValidacion } from "@/types/error_validacion";
import type { RegistrarTasaRequest, TasaDTO } from "@/types/tasas";
import "../accounts.css";

function today() {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function formatAmount(value: string, currency?: Currency) {
  const decimals = Math.min(currency?.decimales ?? 8, 8);
  const [integerPart, fractionPart] = new Decimal(value)
    .toFixed(decimals)
    .split(".");
  const integer = new Intl.NumberFormat("es-VE", {
    maximumFractionDigits: 0,
  }).format(BigInt(integerPart));

  if (decimals === 0) return integer;

  const decimalSeparator =
    new Intl.NumberFormat("es-VE")
      .formatToParts(1.1)
      .find((part) => part.type === "decimal")?.value ?? ",";

  return `${integer}${decimalSeparator}${fractionPart}`;
}

function errorMessage(error: unknown) {
  const validationError = parseErrorValidacion(error);
  if (validationError) return validationError.mensaje;
  if (typeof error === "string") return error;
  if (
    typeof error === "object" &&
    error !== null &&
    "mensaje" in error &&
    typeof error.mensaje === "string"
  ) {
    return error.mensaje;
  }
  return error instanceof Error ? error.message : "Ocurrió un error inesperado.";
}

export default function TasasPage() {
  const [date, setDate] = useState(today);
  const [currencies, setCurrencies] = useState<Currency[]>([]);
  const [rates, setRates] = useState<TasaDTO[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  const loadRates = useCallback(async (cutoff: string): Promise<boolean> => {
    setLoading(true);
    setError("");
    try {
      const history = await listRateHistory(cutoff);
      setRates(history);
      return true;
    } catch (loadError) {
      console.error("No se pudo cargar el historial de tasas.", loadError);
      setError(errorMessage(loadError));
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void listCurrencies()
      .then(setCurrencies)
      .catch((loadError: unknown) => {
        console.error("No se pudo cargar el catálogo de monedas.", loadError);
        setError("No se pudo cargar el catálogo de monedas.");
      });
  }, []);

  useEffect(() => {
    const request = window.setTimeout(() => {
      void loadRates(date);
    }, 0);
    return () => window.clearTimeout(request);
  }, [date, loadRates]);

  async function handleRegister(request: RegistrarTasaRequest) {
    setSaving(true);
    setError("");
    setNotice("");
    try {
      await registrarTasa(request);
      const historyUpdated = await loadRates(date);
      setNotice(
        historyUpdated
          ? "Tasa registrada y agregada al historial."
          : "Tasa registrada, pero no se pudo actualizar el historial.",
      );
    } catch (registerError) {
      console.error("No se pudo registrar la tasa.", registerError);
      throw registerError;
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(rate: TasaDTO) {
    if (
      rate.eliminada ||
      !window.confirm(
        `¿Eliminar la tasa ${rate.moneda_origen} → ${rate.moneda_destino} del ${rate.fecha}?`,
      )
    ) {
      return;
    }

    setDeletingId(rate.id);
    setError("");
    setNotice("");
    try {
      await deleteRate(rate.id);
      const historyUpdated = await loadRates(date);
      setNotice(
        historyUpdated
          ? "Tasa eliminada; su registro se conserva en el historial."
          : "Tasa eliminada, pero no se pudo actualizar el historial.",
      );
    } catch (deleteError) {
      console.error("No se pudo eliminar la tasa.", deleteError);
      setError(errorMessage(deleteError));
    } finally {
      setDeletingId(null);
    }
  }

  const groupedRates = useMemo(() => {
    const groups = new Map<string, TasaDTO[]>();
    for (const rate of rates) {
      const key = `${rate.moneda_origen}:${rate.moneda_destino}`;
      groups.set(key, [...(groups.get(key) ?? []), rate]);
    }
    return [...groups.entries()];
  }, [rates]);

  return (
    <div className="accounts-page patrimonio-page">
      <div className="accounts-shell">
        <AccountsSidebar active="tasas" />
        <main className="accounts-workspace">
          <header className="accounts-header tasas-header">
            <div>
              <p className="breadcrumb">
                Inicio <span>/</span> Tasas de cambio
              </p>
              <h1>Tasas de cambio</h1>
              <p>Registra tasas y consulta el historial de cada par de monedas.</p>
            </div>
            <label className="patrimonio-date">
              Historial hasta
              <input
                type="date"
                value={date}
                max={today()}
                onChange={(event) => setDate(event.currentTarget.value)}
              />
            </label>
          </header>

          {error && (
            <p className="accounts-feedback tasas-error" role="alert">
              {error}
            </p>
          )}
          {notice && (
            <p className="accounts-feedback tasas-notice" role="status">
              {notice}
            </p>
          )}

          <RegistrarTasaForm
            key={date}
            currencies={currencies}
            initialDate={date}
            onRegister={handleRegister}
          />

          {saving && <p role="status">Registrando tasa…</p>}
          {loading ? (
            <p className="accounts-feedback">Cargando historial de tasas…</p>
          ) : groupedRates.length === 0 ? (
            <section className="tasas-empty">
              <h2>Sin tasas registradas</h2>
              <p>
                Registra una tasa para ver aquí el historial del par de monedas.
              </p>
            </section>
          ) : (
            <div className="tasas-history">
              {groupedRates.map(([pair, pairRates]) => {
                const [origin, destination] = pair.split(":");
                const originCurrency = currencies.find(
                  (currency) => currency.codigo === origin,
                );
                const destinationCurrency = currencies.find(
                  (currency) => currency.codigo === destination,
                );

                return (
                  <section className="tasas-pair" key={pair}>
                    <header>
                      <div>
                        <h2>
                          {origin} → {destination}
                        </h2>
                        <p>
                          {originCurrency?.nombre ?? origin} a{" "}
                          {destinationCurrency?.nombre ?? destination}
                        </p>
                      </div>
                      <span>{pairRates.length} registros</span>
                    </header>
                    <div className="patrimonio-table-wrap">
                      <table className="tasas-table">
                        <thead>
                          <tr>
                            <th>Fecha</th>
                            <th>Valor (1 {origin})</th>
                            <th>Fuente</th>
                            <th>Estado</th>
                            <th>Acción</th>
                          </tr>
                        </thead>
                        <tbody>
                          {pairRates.map((rate) => (
                            <tr key={rate.id}>
                              <td>{rate.fecha}</td>
                              <td>
                                {formatAmount(rate.valor, destinationCurrency)}{" "}
                                {destination}
                              </td>
                              <td>{rate.fuente}</td>
                              <td>
                                <span
                                  className={
                                    rate.eliminada
                                      ? "tasas-status tasas-status-deleted"
                                      : "tasas-status"
                                  }
                                >
                                  {rate.eliminada ? "Eliminada" : "Activa"}
                                </span>
                              </td>
                              <td>
                                <button
                                  className="tasas-delete-button"
                                  type="button"
                                  disabled={
                                    rate.eliminada || deletingId === rate.id
                                  }
                                  onClick={() => void handleDelete(rate)}
                                >
                                  {deletingId === rate.id
                                    ? "Eliminando…"
                                    : "Eliminar"}
                                </button>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </section>
                );
              })}
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
