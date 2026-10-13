"use client";
import { useEffect, useState } from "react";
import {
  accountError,
  createAccount,
  isAccountsPreview,
  listAccounts,
  listCurrencies,
} from "@/services/accountsService";
import type {
  Account,
  AccountError,
  CreateAccountInput,
  Currency,
} from "@/types/accounts";
export function useAccounts() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [currencies, setCurrencies] = useState<Currency[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [preview, setPreview] = useState(false);
  const [loadVersion, setLoadVersion] = useState(0);
  useEffect(() => {
    let active = true;
    Promise.all([listAccounts(), listCurrencies()])
      .then(([nextAccounts, nextCurrencies]) => {
        if (!active) return;
        setPreview(isAccountsPreview());
        setAccounts(nextAccounts);
        setCurrencies(nextCurrencies);
      })
      .catch((cause) => {
        if (active)
          setError(
            accountError(
              cause,
              "No pudimos cargar tus cuentas. Vuelve a intentarlo; si el problema continúa, comprueba que esta versión incluye el módulo local de cuentas.",
            ).mensaje,
          );
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [loadVersion]);
  function reload() {
    setLoading(true);
    setError(undefined);
    setLoadVersion((current) => current + 1);
  }
  async function create(
    input: CreateAccountInput,
  ): Promise<AccountError | undefined> {
    try {
      const account = await createAccount(input);
      setAccounts((current) => [
        ...current.filter((item) => item.id !== account.id),
        account,
      ]);
      return undefined;
    } catch (cause) {
      return accountError(
        cause,
        "No pudimos guardar la cuenta. Inténtalo de nuevo.",
      );
    }
  }
  return { accounts, currencies, loading, error, preview, reload, create };
}
