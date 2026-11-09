"use client";

import { useEffect, useState } from "react";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { createCurrency } from "@/services/createCurrency";
import { listCurrencies } from "@/services/listCurrencies";
import type { Currency } from "@/types/accounts";
import { GlobalForm, FormField } from "@/components/forms";
import { createCurrencySchema, type CreateCurrencyFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";
import type { UseFormReturn } from "react-hook-form";
import "../accounts.css";

export default function MonedasPage() {
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

  async function submit(values: CreateCurrencyFormValues, form: UseFormReturn<CreateCurrencyFormValues>) {
    setError("");
    setNotice("");
    try {
      const currency = await createCurrency(values);
      setCurrencies((current) => [...current, currency]);
      setNotice(`Moneda ${currency.codigo} creada correctamente.`);
      form.reset();
    } catch (cause) {
      console.error("No se pudo crear la moneda.", cause);
      if (!applyBackendValidationError(cause, form.setError)) {
        setError("No se pudo crear la moneda.");
      }
    }
  }

  return (
    <div className="accounts-page patrimonio-page">
      <div className="accounts-shell">
        <AccountsSidebar active="monedas" />
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
            <GlobalForm<CreateCurrencyFormValues>
              schema={createCurrencySchema}
              defaultValues={{ codigo: "", nombre: "", simbolo: "", decimales: 2 }}
              onSubmit={submit}
            >
              {(form) => (
                <>
                  <FormField<CreateCurrencyFormValues> name="codigo" label="Código" help="3 a 5 letras mayúsculas; debe ser único.">
                    {(props) => <input {...props} autoCapitalize="characters" autoComplete="off" {...form.register("codigo")} />}
                  </FormField>
                  <FormField<CreateCurrencyFormValues> name="nombre" label="Nombre">
                    {(props) => <input {...props} {...form.register("nombre")} />}
                  </FormField>
                  <FormField<CreateCurrencyFormValues> name="simbolo" label="Símbolo">
                    {(props) => <input {...props} {...form.register("simbolo")} />}
                  </FormField>
                  <FormField<CreateCurrencyFormValues> name="decimales" label="Decimales">
                    {(props) => <input {...props} type="number" {...form.register("decimales")} />}
                  </FormField>
                  <button className="account-primary" type="submit" disabled={form.formState.isSubmitting}>
                    {form.formState.isSubmitting ? "Guardando…" : "Crear moneda"}
                  </button>
                </>
              )}
            </GlobalForm>
          </section>

          <section
            className="currency-catalog"
            aria-labelledby="currency-catalog-title"
          >
            <h2 id="currency-catalog-title">Catálogo disponible</h2>
            {loading ? (
              <p className="accounts-feedback">Cargando monedas…</p>
            ) : (
              <div className="patrimonio-table-wrap">
                <table className="tasas-table">
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
