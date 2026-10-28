import Decimal from "decimal.js";

export function minorUnits(amount: string): Decimal {
  return new Decimal(amount);
}

export function clearAmountInput(value: string): string {
  const normalized = value.replace(",", ".").replace(/[^\d.]/g, "");
  const separatorIndex = normalized.indexOf(".");
  if (separatorIndex < 0) return normalized;

  const whole = normalized.slice(0, separatorIndex);
  const fraction = normalized.slice(separatorIndex + 1).replace(/\./g, "");
  return `${whole}.${fraction}`;
}
