"use client";

import { useEffect, useState } from "react";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { CurrencyForm } from "@/components/currencies/CurrencyForm";
import { crearMonedaDesdeFormulario } from "@/features/currencies/create-currency";
import { useAuthSession } from "@/features/auth/useAuthSession";
import { listCurrencies } from "@/services/listCurrencies";
import type { Currency } from "@/types/accounts";
import type { CreateCurrencyFormValues } from "@/types/schemas";
import type { UseFormReturn } from "react-hook-form";
import "./monedas.css";

export default function MonedasPage() {
  const { usuario } = useAuthSession();
  const [currencies, setCurrencies] = useState<Currency[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(""), 10000);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  useEffect(() => {
    void listCurrencies()
      .then(setCurrencies)
      .catch((cause: unknown) => {
        console.error("No se pudo cargar el catálogo de monedas.", cause);
        setError("No se pudo cargar el catálogo de monedas.");
      })
      .finally(() => setLoading(false));
  }, []);

  function submit(
    values: CreateCurrencyFormValues,
    form: UseFormReturn<CreateCurrencyFormValues>,
  ) {
    setError("");
    setNotice("");
    return crearMonedaDesdeFormulario(
      values,
      form,
      setError,
      (currency) => {
        setCurrencies((current) => [...current, currency]);
        setNotice(`Moneda ${currency.codigo} creada correctamente.`);
      },
    );
  }

  return (
    <div className="accounts-page patrimonio-page">
      <div className="accounts-shell">
        <AccountsSidebar active="monedas" sesionActiva={Boolean(usuario)} />
        <main className="accounts-workspace">
          <header className="accounts-header tasas-header">
            <div>
              <p className="breadcrumb">
                Inicio <span>/</span> Monedas
              </p>
              <h1>Monedas</h1>
              <p>
                El catálogo también incluye las monedas de fábrica, que no se
                pueden modificar.
              </p>
            </div>
          </header>

          <section className="currency-create-panel" aria-labelledby="create-currency-title">
            <h2 id="create-currency-title">Agregar moneda propia</h2>
            {error && (
              <p className="accounts-feedback tasas-error" role="alert">
                {error}
              </p>
            )}
            {notice && (
              <p className="floating-notice" role="status">
                {notice}
              </p>
            )}
            <CurrencyForm onSubmit={submit} />
          </section>

          <section
            className="currency-catalog"
            aria-labelledby="currency-catalog-title"
          >
            <h2 id="currency-catalog-title">Catálogo disponible</h2>
            {loading ? (
              <p className="accounts-feedback">Cargando monedas…</p>
            ) : (
              <div className="currency-catalog-table-wrap">
                <table className="currency-catalog-table">
                  <thead>
                    <tr>
                      <th>Código</th>
                      <th>Nombre</th>
                      <th>Símbolo</th>
                      <th>Decimales</th>
                      <th>Tipo</th>
                    </tr>
                  </thead>
                  <tbody>
                    {currencies.map((currency) => (
                      <tr key={currency.codigo}>
                        <td>{currency.codigo}</td>
                        <td>{currency.nombre}</td>
                        <td>{currency.simbolo}</td>
                        <td>{currency.decimales}</td>
                        <td>
                          {currency.es_del_catalogo ? "De fábrica" : "Propia"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            <p className="currency-catalog-hint">
              Las monedas del catálogo se pueden elegir al crear una cuenta.
            </p>
          </section>
        </main>
      </div>
    </div>
  );
}
