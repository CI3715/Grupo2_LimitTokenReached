export type AccountType =
  | "efectivo"
  | "banco"
  | "billetera_digital";
export type CurrencyCode = string;
import type { CreateAccountFormValues } from "./schemas";

// Decimal strings preserve the precision of amounts displayed in the UI.
export type Account = {
  id: string;
  nombre: string;
  tipo: AccountType;
  moneda_codigo: CurrencyCode;
  saldo: string;
  creada_en: string;
};
export type Currency = {
  codigo: CurrencyCode;
  nombre: string;
  simbolo: string;
  decimales: number;
  es_del_catalogo: boolean;
};
export type CreateAccountInput = CreateAccountFormValues;

export const accountTypeLabels: Record<AccountType, string> = {
  efectivo: "Efectivo",
  banco: "Banco",
  billetera_digital: "Billetera digital",
};
