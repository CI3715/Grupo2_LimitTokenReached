// Espejo de crates/contrato/src/movimiento.rs. Los importes viajan como texto.
export type TipoMovimientoDTO = "Ingreso" | "Gasto" | "Ajuste";
export type TipoRegistroMovimiento = Exclude<TipoMovimientoDTO, "Ajuste">;

export interface CategoriaDTO {
  id: string;
  nombre: string;
  tipo: TipoMovimientoDTO;
}

export interface RegistrarMovimientoRequest {
  cuenta_id: string;
  tipo: TipoMovimientoDTO;
  monto: string;
  fecha: string;
  categoria_id: string;
  nota: string | null;
  confirmar_saldo_negativo?: boolean;
}

export interface CorregirMovimientoRequest {
  movimiento_id: string;
  monto_correcto: string;
  confirmar_saldo_negativo?: boolean;
}

export interface MovimientoDTO extends RegistrarMovimientoRequest {
  id: string;
  es_correccion: boolean;
  movimiento_original_id: string | null;
}

export interface MovementFormError {
  campo?: string;
  mensaje: string;
  // Una respuesta inesperada puede llegar después de que el comando haya guardado.
  verificarRegistro?: boolean;
}

// Comparación exacta exclusivamente para la advertencia de saldo negativo.
// No valida las reglas del dominio ni calcula el saldo persistido.
export function expenseExceedsBalance(amount: string, balance: string): boolean {
  const decimal = /^-?\d+(?:\.\d+)?$/;
  if (!decimal.test(amount) || !decimal.test(balance)) return false;
  const precision = Math.max(
    amount.split(".")[1]?.length ?? 0,
    balance.split(".")[1]?.length ?? 0,
  );
  function scaled(value: string): bigint {
    const negative = value.startsWith("-");
    const [whole, fraction = ""] = value.replace(/^-/, "").split(".");
    const units = BigInt(whole + fraction.padEnd(precision, "0"));
    return negative ? -units : units;
  }
  return scaled(amount) > scaled(balance);
}
