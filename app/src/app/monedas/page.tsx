"use client";

import { useEffect, useState, type FormEvent } from "react";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { createCurrency } from "@/services/createCurrency";
import { listCurrencies } from "@/services/listCurrencies";
import type { Currency } from "@/types/accounts";
import {
  parseErrorValidacion,
  type ErrorValidacionDTO,
} from "@/types/error_validacion";
import "../accounts.css";

export default function MonedasPage() {
  const [currencies, setCurrencies] = useState<Currency[]>([]);
  const [error, setError] = useState<ErrorValidacionDTO | null>(null);
  const [notice, setNotice] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    void listCurrencies()
      .then(setCurrencies)
      .catch((cause: unknown) => {
        console.error("No se pudo cargar el catálogo de monedas.", cause);
        setError({
          campo: "general",
          mensaje: "No se pudo cargar el catálogo de monedas.",
        });
      })
      .finally(() => setLoading(false));
  }, []);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setNotice("");
    setSaving(true);
    const formElement = event.currentTarget;
    const form = new FormData(formElement);

    try {
      const currency = await createCurrency({
        codigo: String(form.get("codigo") ?? "").trim().toUpperCase(),
        nombre: String(form.get("nombre") ?? "").trim(),
        simbolo: String(form.get("simbolo") ?? "").trim(),
        decimales: Number(form.get("decimales")),
      });
      setCurrencies((current) => [...current, currency]);
      setNotice(`Moneda ${currency.codigo} creada correctamente.`);
      formElement.reset();
    } catch (cause) {
      console.error("No se pudo crear la moneda.", cause);
      setError(
        parseErrorValidacion(cause) ?? {
          campo: "general",
          mensaje: "No se pudo crear la moneda.",
        },
      );
    } finally {
      setSaving(false);
    }
  }

  function fieldError(field: string) {
    return error?.campo === field ? error.mensaje : undefined;
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
            {error?.campo === "general" && (
              <p className="accounts-feedback tasas-error" role="alert">
                {error.mensaje}
              </p>
            )}
            {notice && (
              <p className="accounts-feedback tasas-notice" role="status">
                {notice}
              </p>
            )}
            <form onSubmit={submit}>
              <label>
                Código
                <input
                  name="codigo"
                  maxLength={5}
                  autoCapitalize="characters"
                  autoComplete="off"
                  required
                  aria-invalid={Boolean(fieldError("codigo"))}
                  aria-describedby={fieldError("codigo") ? "codigo-error" : undefined}
                  onChange={(event) => {
                    event.currentTarget.value =
                      event.currentTarget.value.toUpperCase();
                  }}
                />
                <small>3 a 5 letras mayúsculas; debe ser único.</small>
                {fieldError("codigo") && (
                  <small id="codigo-error" className="account-error" role="alert">
                    {fieldError("codigo")}
                  </small>
                )}
              </label>
              <label>
                Nombre
                <input
                  name="nombre"
                  required
                  aria-invalid={Boolean(fieldError("nombre"))}
                  aria-describedby={fieldError("nombre") ? "nombre-error" : undefined}
                />
                {fieldError("nombre") && (
                  <small id="nombre-error" className="account-error" role="alert">
                    {fieldError("nombre")}
                  </small>
                )}
              </label>
              <label>
                Símbolo
                <input
                  name="simbolo"
                  required
                  aria-invalid={Boolean(fieldError("simbolo"))}
                  aria-describedby={fieldError("simbolo") ? "simbolo-error" : undefined}
                />
                {fieldError("simbolo") && (
                  <small id="simbolo-error" className="account-error" role="alert">
                    {fieldError("simbolo")}
                  </small>
                )}
              </label>
              <label>
                Decimales
                <input
                  name="decimales"
                  type="number"
                  min={0}
                  max={8}
                  step={1}
                  defaultValue={2}
                  required
                  aria-invalid={Boolean(fieldError("decimales"))}
                  aria-describedby={
                    fieldError("decimales") ? "decimales-error" : undefined
                  }
                />
                {fieldError("decimales") && (
                  <small id="decimales-error" className="account-error" role="alert">
                    {fieldError("decimales")}
                  </small>
                )}
              </label>
              <button className="account-primary" type="submit" disabled={saving}>
                {saving ? "Guardando…" : "Crear moneda"}
              </button>
            </form>
          </section>

          <section className="currency-catalog" aria-labelledby="currency-catalog-title">
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
