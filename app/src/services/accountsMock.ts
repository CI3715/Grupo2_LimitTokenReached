import type {
  Account,
  AccountError,
  CreateAccountInput,
  Currency,
} from "@/types/accounts";

export const demoCurrencies: Currency[] = [
  { codigo: "VES", nombre: "Bolívar", simbolo: "Bs.", decimales: 2 },
  { codigo: "USD", nombre: "Dólar estadounidense", simbolo: "$", decimales: 2 },
  { codigo: "EUR", nombre: "Euro", simbolo: "€", decimales: 2 },
  { codigo: "USDT", nombre: "Tether", simbolo: "USDT", decimales: 2 },
];
let accounts: Account[] = [
  {
    id: "demo-ves",
    nombre: "Efectivo diario",
    tipo: "efectivo",
    moneda: "VES",
    saldo: "12500.00",
  },
  {
    id: "demo-usd",
    nombre: "Cuenta dólares",
    tipo: "banco",
    moneda: "USD",
    saldo: "340.00",
  },
  {
    id: "demo-usdt",
    nombre: "Ahorros USDT",
    tipo: "billetera",
    moneda: "USDT",
    saldo: "180.00",
  },
];
export async function listDemoAccounts() {
  return accounts.map((account) => ({ ...account }));
}
export async function createDemoAccount(
  input: CreateAccountInput,
): Promise<Account> {
  // Only the isolated preview adapter simulates domain responses. No persistence.
  const fail = (error: AccountError): never => {
    throw error;
  };
  const name = input.nombre.trim();
  if (!name)
    fail({ campo: "nombre", mensaje: "Escribe un nombre para la cuenta." });
  if (
    accounts.some(
      (account) =>
        account.nombre.toLocaleLowerCase("es") === name.toLocaleLowerCase("es"),
    )
  )
    fail({ campo: "nombre", mensaje: "Ya existe una cuenta con ese nombre." });
  if (!demoCurrencies.some((currency) => currency.codigo === input.moneda))
    fail({ campo: "moneda", mensaje: "Selecciona una moneda del catálogo." });
  if (!/^\d+(?:\.\d{1,2})?$/.test(input.saldo_inicial))
    fail({
      campo: "saldo_inicial",
      mensaje: "Indica un saldo no negativo con hasta dos decimales.",
    });
  const account: Account = {
    id: crypto.randomUUID(),
    nombre: name,
    tipo: input.tipo,
    moneda: input.moneda,
    saldo: input.saldo_inicial,
  };
  accounts = [...accounts, account];
  return { ...account };
}
