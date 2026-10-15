"use client";

import { useState } from "react";
import type { Account, CreateAccountInput } from "@/types/accounts";
import { createAccount } from "@/services/createAcount";
import { demoCurrencies } from "./mock-data";

export function useAccounts() {
  const [accounts, setAccounts] = useState<Account[]>([]);

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