export type AccountType =
  | "efectivo"
  | "banco"
  | "billetera_digital";
export type CurrencyCode = string;

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
export type CreateAccountInput = {
  nombre: string;
  tipo: AccountType;
  moneda_codigo: CurrencyCode;
  saldo_inicial: string;
};

export const accountTypeLabels: Record<AccountType, string> = {
  efectivo: "Efectivo",
  banco: "Banco",
  billetera_digital: "Billetera digital",
};
