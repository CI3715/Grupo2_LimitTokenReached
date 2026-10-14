"use client";
import { useState } from "react";
import type { Account, CreateAccountInput } from "@/types/accounts";
import { demoCurrencies, initialAccounts } from "./mock-data";

// UI preview only: changes live in React state and reset on reload.
export function useAccounts() {
  const [accounts, setAccounts] = useState<Account[]>(initialAccounts);
  function create(input: CreateAccountInput) {
    const account: Account = {
      id: crypto.randomUUID(),
      nombre: input.nombre,
      tipo: input.tipo,
      moneda: input.moneda,
      saldo: input.saldo_inicial,
    };
    setAccounts((current) => [...current, account]);
  }
  return { accounts, currencies: demoCurrencies, create };
}
