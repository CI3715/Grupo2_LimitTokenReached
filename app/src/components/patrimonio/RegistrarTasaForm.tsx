"use client";

import { useState, type FormEvent } from "react";
import type { Currency } from "@/types/accounts";
import {
  parseErrorValidacion,
  type ErrorValidacionDTO,
} from "@/types/error_validacion";
import type { RegistrarTasaRequest } from "@/types/tasas";

type Props = {
  currencies: Currency[];
  initialOrigin?: string;
  initialDestination?: string;
  initialDate: string;
  onRegister: (request: RegistrarTasaRequest) => Promise<void>;
};

function today() {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

export function RegistrarTasaForm({
  currencies,
  initialOrigin,
  initialDestination,
  initialDate,
  onRegister,
}: Props) {
  const [origin, setOrigin] = useState("");
  const [destination, setDestination] = useState("");
  const [error, setError] = useState<ErrorValidacionDTO | null>(null);
  const [saving, setSaving] = useState(false);

  const selectedOrigin = currencies.some(
    (currency) => currency.codigo === origin,
  )
    ? origin
    : currencies.some((currency) => currency.codigo === initialOrigin)
      ? initialOrigin!
      : currencies[0]?.codigo ?? "";
  const selectedDestination =
    currencies.some(
      (currency) =>
        currency.codigo === destination &&
        currency.codigo !== selectedOrigin,
    )
      ? destination
      : currencies.some(
            (currency) =>
              currency.codigo === initialDestination &&
              currency.codigo !== selectedOrigin,
          )
        ? initialDestination!
        : currencies.find(
            (currency) => currency.codigo !== selectedOrigin,
          )?.codigo ?? "";

  function fieldError(...fields: string[]) {
    return fields.includes(error?.campo ?? "") ? error?.mensaje : undefined;
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);

    if (selectedOrigin === selectedDestination) {
      setError({
        campo: "moneda_destino",
        mensaje: "Selecciona dos monedas diferentes.",
      });
      return;
    }

    const form = new FormData(event.currentTarget);
    setSaving(true);

    try {
      await onRegister({
        moneda_origen: selectedOrigin,
        moneda_destino: selectedDestination,
        valor: String(form.get("valor") ?? "").trim().replace(",", "."),
        fecha: String(form.get("fecha") ?? ""),
        fuente: String(form.get("fuente") ?? "").trim(),
      });

    } catch (cause) {
      const validationError = parseErrorValidacion(cause);
      setError(validationError ?? {
        campo: "",
        mensaje:
          cause instanceof Error
            ? cause.message
            : typeof cause === "string"
              ? cause
              : "No se pudo registrar la tasa.",
      });
    } finally {
      setSaving(false);
    }
  }

  return (
    <section
      className="patrimonio-rate-form"
      id="registrar-tasa-form"
      aria-labelledby="registrar-tasa-title"
    >
      <h2 id="registrar-tasa-title">Registrar tasa de cambio</h2>

      {error &&
        ![
          "moneda",
          "moneda_origen",
          "moneda_destino",
          "valor",
          "fecha",
          "fuente",
        ].includes(error.campo) && (
          <p role="alert">{error.mensaje}</p>
        )}
      <form onSubmit={submit}>
        <label>
          Moneda de origen
          <select
            value={selectedOrigin}
            onChange={(event) => setOrigin(event.currentTarget.value)}
            required
            disabled={saving || currencies.length === 0}
          >
            {currencies.map((currency) => (
              <option key={currency.codigo} value={currency.codigo}>
                {currency.codigo} · {currency.nombre}
              </option>
            ))}
          </select>
          {fieldError("moneda", "moneda_origen") && (
            <small role="alert">{fieldError("moneda", "moneda_origen")}</small>
          )}
        </label>

        <label>
          Moneda de destino
          <select
            value={selectedDestination}
            onChange={(event) => setDestination(event.currentTarget.value)}
            required
            disabled={saving || currencies.length < 2}
          >
            {currencies.map((currency) => (
              <option
                key={currency.codigo}
                value={currency.codigo}
                disabled={currency.codigo === selectedOrigin}
              >
                {currency.codigo} · {currency.nombre}
              </option>
            ))}
          </select>
          {fieldError("moneda", "moneda_destino") && (
            <small role="alert">
              {fieldError("moneda", "moneda_destino")}
            </small>
          )}
        </label>

        <label>
          Valor (unidades de destino por una de origen)
          <input
            name="valor"
            type="text"
            inputMode="decimal"
            pattern="[0-9]+([.,][0-9]{1,8})?"
            placeholder="Ej. 887.00"
            required
            disabled={saving}
          />
          {fieldError("valor") && (
            <small role="alert">{fieldError("valor")}</small>
          )}
        </label>

        <label>
          Fecha
          <input
            name="fecha"
            type="date"
            defaultValue={initialDate}
            max={today()}
            required
            disabled={saving}
          />
          {fieldError("fecha") && (
            <small role="alert">{fieldError("fecha")}</small>
          )}
        </label>

        <label>
          Fuente
          <input
            name="fuente"
            type="text"
            placeholder="Ej. BCV"
            required
            disabled={saving}
          />
          {fieldError("fuente") && (
            <small role="alert">{fieldError("fuente")}</small>
          )}
        </label>

        <button type="submit" disabled={saving || currencies.length < 2}>
          {saving ? "Registrando…" : "Registrar tasa"}
        </button>
      </form>
    </section>
  );
}