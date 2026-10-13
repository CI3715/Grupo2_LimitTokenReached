// Presentation arithmetic in integer minor units; Rust owns domain validation.
export function minorUnits(amount: string): bigint {
  if (!/^-?\d+(?:\.\d{1,2})?$/.test(amount)) throw new Error("Monto no válido");
  const negative = amount.startsWith("-");
  const [whole, fraction = ""] = amount.replace(/^-/, "").split(".");
  const result = BigInt(whole) * BigInt(100) + BigInt(fraction.padEnd(2, "0"));
  return negative ? -result : result;
}
export function formatMoney(amount: bigint): string {
  const negative = amount < BigInt(0);
  const absolute = negative ? -amount : amount;
  const whole = (absolute / BigInt(100))
    .toString()
    .replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  return `${negative ? "−" : ""}${whole},${(absolute % BigInt(100)).toString().padStart(2, "0")}`;
}
