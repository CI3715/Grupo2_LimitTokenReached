import { invoke } from "@tauri-apps/api/core";
import type { Account } from "@/types/accounts";

export async function listAccounts(): Promise<Account[]> {
  return invoke<Account[]>("listar_cuentas");
}
