import Decimal from "decimal.js";

export function minorUnits(amount: string): Decimal {
  return new Decimal(amount);
}

export function sanitizeAmountInput(value: string, decimalPlaces: number): string {
  const normalized = value.replace(",", ".").replace(/[^\d.]/g, "");
  const separatorIndex = normalized.indexOf(".");
  if (separatorIndex < 0) return normalized;

  const whole = normalized.slice(0, separatorIndex);
  const fraction = normalized
    .slice(separatorIndex + 1)
    .replace(/\./g, "")
    .slice(0, decimalPlaces);
  return `${whole}.${fraction}`;
}

export function sanitizeFreeAmountInput(value: string): string {
  const normalized = value.replace(",", ".").replace(/[^\d.-]/g, "");
  const sign = normalized.startsWith("-") ? "-" : "";
  const unsigned = normalized.replace(/-/g, "");
  const separatorIndex = unsigned.indexOf(".");
  if (separatorIndex < 0) return `${sign}${unsigned}`;

  const whole = unsigned.slice(0, separatorIndex);
  const fraction = unsigned.slice(separatorIndex + 1).replace(/\./g, "");
  return `${sign}${whole}.${fraction}`;
}
