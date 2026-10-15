export type AccountType = "efectivo" | "banco" | "billetera";
export type CurrencyCode = "VES" | "USD" | "EUR" | "USDT";

// Decimal strings preserve the precision of amounts displayed in the UI.
export type Account = {
  id: string;
  nombre: string;
  tipo: AccountType;
  moneda: CurrencyCode;
  saldo: string;
};
export type Currency = {
  codigo: CurrencyCode;
  nombre: string;
  simbolo: string;
  decimales: number;
};
export type CreateAccountInput = Omit<Account, "id" | "saldo"> & {
  saldo_inicial: string;
};
export const accountTypeLabels: Record<AccountType, string> = {
  efectivo: "Efectivo",
  banco: "Banco",
  billetera: "Billetera digital",
};
