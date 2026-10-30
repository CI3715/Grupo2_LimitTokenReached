"use client";

import Decimal from "decimal.js";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { RegistrarTasaForm } from "@/components/patrimonio/RegistrarTasaForm";
import { changeBaseCurrency } from "@/services/changeBaseCurrency";
import { consultarPatrimonio } from "@/services/consultarPatrimonio";
import { listCurrencies } from "@/services/listCurrencies";
import { registrarTasa } from "@/services/registrarTasa";
import type { Currency } from "@/types/accounts";
import type { Patrimonio, SaldoCuenta, TasaUsada } from "@/types/patrimonio";
import type { RegistrarTasaRequest } from "@/types/tasas";
import "../accounts.css";

function today() {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function formatAmount(value: string | Decimal, currency?: Currency) {
  const decimals = currency?.decimales ?? 2;
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

function currencyName(code: string, currencies: Currency[]) {
  const currency = currencies.find((item) => item.codigo === code);
  return currency ? `${currency.nombre} (${code})` : code;
}

function rateFor(
  accountCurrency: string,
  baseCurrency: string,
  rates: TasaUsada[],
) {
  return rates.find(
    (rate) =>
      (rate.moneda_origen === accountCurrency &&
        rate.moneda_destino === baseCurrency &&
        !rate.es_inversa) ||
      (rate.moneda_origen === baseCurrency &&
        rate.moneda_destino === accountCurrency &&
        rate.es_inversa),
  );
}

function effectiveRate(rate: TasaUsada) {
  return rate.es_inversa
    ? new Decimal(1).div(rate.valor)
    : new Decimal(rate.valor);
}

function convertedBalance(
  account: SaldoCuenta,
  baseCurrency: string,
  rates: TasaUsada[],
) {
  if (account.moneda === baseCurrency) return new Decimal(account.saldo);
  const rate = rateFor(account.moneda, baseCurrency, rates);
  return rate ? new Decimal(account.saldo).mul(effectiveRate(rate)) : null;
}

function RateCell({
  accountCurrency,
  baseCurrency,
  rates,
  date,
}: {
  accountCurrency: string;
  baseCurrency: string;
  rates: TasaUsada[];
  date: string;
}) {
  if (accountCurrency === baseCurrency) {
    return (
      <span className="patrimonio-rate-content patrimonio-rate-muted">
        Moneda base
      </span>
    );
  }

  const rate = rateFor(accountCurrency, baseCurrency, rates);
  if (!rate) {
    return (
      <span className="patrimonio-rate-content patrimonio-rate-missing">
        Falta tasa {accountCurrency} → {baseCurrency} al {date}
      </span>
    );
  }

  return (
    <span className="patrimonio-rate-content">
      1 {accountCurrency} = {formatAmount(effectiveRate(rate))}{" "}
      {baseCurrency}
      <small>
        {rate.fecha} · {rate.fuente}
        {rate.es_inversa ? " · inversa" : ""}
      </small>
    </span>
  );
}

function PatrimonioTable({
  currencyCode,
  accounts,
  data,
  currencies,
  date,
}: {
  currencyCode: string;
  accounts: SaldoCuenta[];
  data: Patrimonio;
  currencies: Currency[];
  date: string;
}) {
  const currency = currencies.find((item) => item.codigo === currencyCode);
  const subtotal =
    data.subtotales_por_moneda.find((item) => item.moneda === currencyCode)
      ?.subtotal ?? "0";
  const convertedSubtotal = accounts.reduce<Decimal | null>((total, account) => {
    const converted = convertedBalance(
      account,
      data.moneda_base,
      data.tasas_usadas,
    );
    if (converted === null) return null;
    return (total ?? new Decimal(0)).add(converted);
  }, new Decimal(0));
  const baseCurrency = currencies.find(
    (item) => item.codigo === data.moneda_base,
  );

  return (
    <section className="patrimonio-currency" aria-labelledby={`currency-${currencyCode}`}>
      <div className="patrimonio-currency-heading">
        <div>
        <h2 id={`currency-${currencyCode}`}>
          {currencyName(currencyCode, currencies)}
        </h2>
        <p>
          {accounts.length} {accounts.length === 1 ? "cuenta" : "cuentas"}
        </p>
        </div>
        <div className="patrimonio-currency-subtotal">
          <span>Subtotal en {currencyCode}</span>
          <strong>
            {currency?.simbolo ?? currencyCode} {formatAmount(subtotal, currency)}
          </strong>
        </div>
      </div>
      <div className="patrimonio-table-wrap">
        <table className="patrimonio-table">
          <thead>
            <tr>
              <th>Cuenta</th>
              <th>Saldo</th>
              <th>Tasa usada</th>
              <th>En {data.moneda_base}</th>
            </tr>
          </thead>
          <tbody>
            {accounts.map((account) => {
              const converted = convertedBalance(
                account,
                data.moneda_base,
                data.tasas_usadas,
              );
              return (
                <tr key={account.cuenta_id}>
                  <td>{account.nombre_cuenta}</td>
                  <td>
                    {currency?.simbolo ?? account.moneda}{" "}
                    {formatAmount(account.saldo, currency)}
                  </td>
                  <td>
                    <RateCell
                      accountCurrency={account.moneda}
                      baseCurrency={data.moneda_base}
                      rates={data.tasas_usadas}
                      date={date}
                    />
                  </td>
                  <td>
                    {converted === null
                      ? "—"
                      : `${baseCurrency?.simbolo ?? data.moneda_base} ${formatAmount(converted, baseCurrency)}`}
                  </td>
                </tr>
              );
            })}
            <tr>
              <th colSpan={3}>Subtotal convertido en {data.moneda_base}</th>
              <th>
                {convertedSubtotal === null
                  ? "—"
                  : `${baseCurrency?.simbolo ?? data.moneda_base} ${formatAmount(convertedSubtotal, baseCurrency)}`}
              </th>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  );
}

export default function PatrimonioPage() {
  const [date, setDate] = useState(today);
  const [data, setData] = useState<Patrimonio | null>(null);
  const [currencies, setCurrencies] = useState<Currency[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [changingBase, setChangingBase] = useState(false);
  const [rateFormOpen, setRateFormOpen] = useState(false);
  const requestId = useRef(0);

  const load = useCallback(async (consultDate: string) => {
    const currentRequest = ++requestId.current;
    setLoading(true);
    setError("");

    try {
      const nextData = await consultarPatrimonio({
        fecha_consulta: consultDate,
      });
      if (currentRequest === requestId.current) setData(nextData);
    } catch (loadError) {
      if (currentRequest !== requestId.current) return;
      console.error("No se pudo consultar el patrimonio.", loadError);
      setError("No se pudo consultar el patrimonio. Intenta nuevamente.");
    } finally {
      if (currentRequest === requestId.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    const request = window.setTimeout(() => {
      void load(date);
    }, 0);
    return () => window.clearTimeout(request);
  }, [date, load]);

  useEffect(() => {
    void listCurrencies()
      .then(setCurrencies)
      .catch((loadError: unknown) => {
        console.error("No se pudo cargar el catálogo de monedas.", loadError);
        setError("No se pudo cargar el catálogo de monedas.");
      });
  }, []);

  async function handleBaseCurrencyChange(codigo: string) {
    if (!codigo || codigo === data?.moneda_base) return;
    setChangingBase(true);
    setError("");

    try {
      await changeBaseCurrency(codigo);
      await load(date);
    } catch (changeError) {
      console.error("No se pudo cambiar la moneda base.", changeError);
      setError("No se pudo cambiar la moneda base. Intenta nuevamente.");
    } finally {
      setChangingBase(false);
    }
  }

  async function handleRegisterRate(request: RegistrarTasaRequest) {
    await registrarTasa(request);
    await load(date);
  }

  const groupedAccounts = useMemo(() => {
    if (!data) return [];
    const groups = new Map<string, SaldoCuenta[]>();
    for (const account of data.saldos_por_cuenta) {
      groups.set(account.moneda, [
        ...(groups.get(account.moneda) ?? []),
        account,
      ]);
    }
    return [...groups.entries()];
  }, [data]);
  const missingRates = useMemo(() => {
    if (!data) return [];

    const missing = [...data.tasas_faltantes];
    const knownPairs = new Set(
      missing.map((rate) => `${rate.moneda_origen}:${rate.moneda_destino}`),
    );

    for (const [currency, accounts] of groupedAccounts) {
      if (currency === data.moneda_base) continue;

      const subtotal = accounts.reduce(
        (total, account) => total.add(account.saldo),
        new Decimal(0),
      );
      if (
        subtotal.isZero() ||
        rateFor(currency, data.moneda_base, data.tasas_usadas) ||
        knownPairs.has(`${currency}:${data.moneda_base}`)
      ) {
        continue;
      }

      missing.push({
        moneda_origen: currency,
        moneda_destino: data.moneda_base,
        fecha_limite: date,
      });
    }

    return missing;
  }, [data, date, groupedAccounts]);
  const hasMissingRates = missingRates.length > 0;

  return (
    <div className="accounts-page patrimonio-page">
      <div className="accounts-shell">
        <AccountsSidebar active="patrimonio" />
        <main className="accounts-workspace">
          <header className="accounts-header patrimonio-header">
            <div>
              <p className="breadcrumb">
                Inicio <span>/</span> Patrimonio
              </p>
              <h1>Patrimonio</h1>
              <p>Consulta el valor de tus cuentas agrupado por moneda.</p>
            </div>
            <label className="patrimonio-date">
              Fecha de consulta
              <input
                type="date"
                value={date}
                max={today()}
                onChange={(event) => setDate(event.currentTarget.value)}
              />
            </label>
            <label className="patrimonio-date">
              Moneda base
              <select
                value={data?.moneda_base ?? ""}
                disabled={!data || changingBase}
                onChange={(event) =>
                  void handleBaseCurrencyChange(event.currentTarget.value)
                }
              >
                {currencies.map((currency) => (
                  <option key={currency.codigo} value={currency.codigo}>
                    {currency.nombre} ({currency.codigo})
                  </option>
                ))}
              </select>
            </label>
          </header>

          {loading && (
            <p className="accounts-feedback">Consultando patrimonio...</p>
          )}
          {!loading && error && (
            <p className="accounts-feedback" role="alert">
              {error}
            </p>
          )}
          {!loading && !error && data && (
            <>
              <div className="patrimonio-summary">
                <span>
                  Moneda base: <strong>{data.moneda_base}</strong>
                </span>
                <span>
                  Fecha: <strong>{date}</strong>
                </span>
              </div>
              <div className="patrimonio-total">
                <strong>Total consolidado en {data.moneda_base}</strong>
                <strong>
                  {hasMissingRates || data.total_consolidado === null
                    ? "—"
                    : `${currencies.find((item) => item.codigo === data.moneda_base)?.simbolo ?? data.moneda_base} ${formatAmount(data.total_consolidado, currencies.find((item) => item.codigo === data.moneda_base))}`}
                </strong>
              </div>

              {hasMissingRates && (
                <div className="patrimonio-warning" role="status">
                  No se puede mostrar el total consolidado, faltan las tasas:{" "}
                  {missingRates
                    .map(
                      (rate) =>
                        `${rate.moneda_origen} → ${rate.moneda_destino} al ${rate.fecha_limite}`,
                    )
                    .join(", ")}.
                </div>
              )}

              {currencies.length > 1 && (
                <div className="patrimonio-rate-actions">
                  <button
                    className="patrimonio-register-rate-button"
                    type="button"
                    aria-expanded={rateFormOpen}
                    aria-controls="registrar-tasa-form"
                    onClick={() => setRateFormOpen((isOpen) => !isOpen)}
                  >
                    {rateFormOpen ? "Cerrar formulario" : "Registrar tasa"}
                  </button>
                  {rateFormOpen && (
                    <RegistrarTasaForm
                      key={date}
                      currencies={currencies}
                      initialOrigin={missingRates[0]?.moneda_origen}
                      initialDestination={missingRates[0]?.moneda_destino}
                      initialDate={date}
                      onRegister={handleRegisterRate}
                    />
                  )}
                </div>
              )}

              <div className="patrimonio-tables-scroll">
                <div className="patrimonio-tables">
                  {groupedAccounts.map(([currency, accounts]) => (
                    <PatrimonioTable
                      key={currency}
                      currencyCode={currency}
                      accounts={accounts}
                      data={data}
                      currencies={currencies}
                      date={date}
                    />
                  ))}
                </div>
              </div>
            </>
          )}
        </main>
      </div>
    </div>
  );
}
