"use client";

import { useEffect, useState } from "react";
import type { Account, CreateAccountInput, Currency } from "@/types/accounts";
import { createAccount } from "@/services/createAcount";
import { listCurrencies } from "@/services/listCurrencies";

type ValidationError = {
  campo: string;
  mensaje: string;
};

function isValidationError(error: unknown): error is ValidationError {
  return (
    typeof error === "object" &&
    error !== null &&
    "campo" in error &&
    "mensaje" in error &&
    typeof error.campo === "string" &&
    typeof error.mensaje === "string"
  );
}

export function useAccounts() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [currencies, setCurrencies] = useState<Currency[]>([]);
  const [announcement, setAnnouncement] = useState("");

  useEffect(() => {
    async function loadCurrencies() {
      try {
        setCurrencies(await listCurrencies());
      } catch (error: unknown) {
        console.error("No se pudo cargar el catálogo de monedas.", error);
        setAnnouncement("No se pudo cargar el catálogo de monedas.");
      }
    }

    void loadCurrencies();
  }, []);

  async function create(input: CreateAccountInput): Promise<Account | null> {
    try {
      const account = await createAccount(input);

      setAccounts((current) => [...current, account]);
      setAnnouncement(`Cuenta ${account.nombre} creada correctamente.`);

      return account;
    } catch (error: unknown) {
      if (isValidationError(error)) {
        setAnnouncement(error.mensaje);
      } else {
        console.error("No se pudo crear la cuenta.", error);
        setAnnouncement("No se pudo crear la cuenta.");
      }

      return null;
    }
  }

  return {
    accounts,
    currencies,
    announcement,
    create,
  };
}