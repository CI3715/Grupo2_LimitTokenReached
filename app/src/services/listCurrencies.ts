import { invoke } from "@tauri-apps/api/core";
import type { Currency } from "@/types/accounts";

export async function listCurrencies(): Promise<Currency[]> {
  return invoke<Currency[]>("listar_monedas");
}
