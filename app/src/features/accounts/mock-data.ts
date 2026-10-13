import type { Account, Currency } from "@/types/accounts";

export const demoCurrencies: Currency[] = [
  { codigo: "VES", nombre: "Bolívar", simbolo: "Bs.", decimales: 2 },
  { codigo: "USD", nombre: "Dólar estadounidense", simbolo: "$", decimales: 2 },
  { codigo: "EUR", nombre: "Euro", simbolo: "€", decimales: 2 },
  { codigo: "USDT", nombre: "Tether", simbolo: "USDT", decimales: 2 },
];
export const initialAccounts: Account[] = [
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
