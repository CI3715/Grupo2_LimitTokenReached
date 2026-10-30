import { invoke } from "@tauri-apps/api/core";
import type { Currency, CurrencyCode } from "@/types/accounts";

export type CreateCurrencyInput = {
  codigo: CurrencyCode;
  nombre: string;
  simbolo: string;
  decimales: number;
};

export async function createCurrency(
  datos: CreateCurrencyInput,
): Promise<Currency> {
  return invoke<Currency>("crear_moneda", { datos });
}
