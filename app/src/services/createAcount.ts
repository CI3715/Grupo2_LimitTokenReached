import { invoke } from "@tauri-apps/api/core";
import type { Account } from "@/types/accounts";
import type { CreateAccountFormValues } from "@/types/schemas";

export async function createAccount(input: CreateAccountFormValues): Promise<Account> {
  return invoke<Account>("crear_cuenta", {datos: input,});
}