export type AccountType = "efectivo" | "banco" | "billetera";
export type CurrencyCode = "VES" | "USD" | "EUR" | "USDT";

// Decimal strings cross the IPC boundary without floating-point conversion.
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
export type AccountField = "nombre" | "tipo" | "moneda" | "saldo_inicial";
export type AccountError = { campo?: AccountField; mensaje: string };
export const accountTypeLabels: Record<AccountType, string> = {
  efectivo: "Efectivo",
  banco: "Banco",
  billetera: "Billetera digital",
};
