import { invoke, isTauri } from "@tauri-apps/api/core";
import type {
  Account,
  AccountError,
  CreateAccountInput,
  Currency,
} from "@/types/accounts";
import {
  createDemoAccount,
  demoCurrencies,
  listDemoAccounts,
} from "./accountsMock";

export const isAccountsPreview = () => !isTauri();
export async function listAccounts(): Promise<Account[]> {
  return isTauri() ? invoke<Account[]>("listar_cuentas") : listDemoAccounts();
}
export async function listCurrencies(): Promise<Currency[]> {
  return isTauri() ? invoke<Currency[]>("listar_monedas") : demoCurrencies;
}
export async function createAccount(
  input: CreateAccountInput,
): Promise<Account> {
  return isTauri()
    ? invoke<Account>("crear_cuenta", { datos: input })
    : createDemoAccount(input);
}
export function accountError(error: unknown, fallback: string): AccountError {
  // Only display structured, user-facing errors, never native traces.
  if (
    typeof error === "object" &&
    error !== null &&
    "mensaje" in error &&
    typeof error.mensaje === "string"
  ) {
    const field = "campo" in error ? error.campo : undefined;
    return {
      mensaje: error.mensaje,
      campo:
        field === "nombre" ||
        field === "tipo" ||
        field === "moneda" ||
        field === "saldo_inicial"
          ? field
          : undefined,
    };
  }
  return { mensaje: fallback };
}
