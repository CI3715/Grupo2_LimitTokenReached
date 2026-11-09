import { invoke } from "@tauri-apps/api/core";
import type { Currency } from "@/types/accounts";
import type { CreateCurrencyFormValues } from "@/types/schemas";

export async function createCurrency(
  datos: CreateCurrencyFormValues,
): Promise<Currency> {
  return invoke<Currency>("crear_moneda", { datos });
}
