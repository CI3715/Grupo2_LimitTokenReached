"use client";

import { useEffect, useState } from "react";
import type { Account, CreateAccountInput } from "@/types/accounts";
import { createAccount } from "@/services/createAcount";
import { listAccounts } from "@/services/listAccounts";
import { demoCurrencies } from "./mock-data";

export function useAccounts() {
  const [accounts, setAccounts] = useState<Account[]>([]);

  useEffect(() => {
    listAccounts()
      .then((cuentas) => {
        setAccounts(cuentas);
      })
      .catch((error) => {
        console.error("No se pudieron cargar las cuentas:", error);
      });
  }, []);
  
  async function create(input: CreateAccountInput) {
    const account = await createAccount(input);

    setAccounts((current) => [...current, account]);

    return account;
  }

  return {
    accounts,
    currencies: demoCurrencies,
    create,
  };
}