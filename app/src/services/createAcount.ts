import { invoke } from "@tauri-apps/api/core";
import type { Account, CreateAccountInput } from "@/types/accounts";

export async function createAccount(input: CreateAccountInput,): Promise<Account> {
  return invoke<Account>("crear_cuenta", {datos: input,});
}