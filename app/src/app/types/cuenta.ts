export type TipoCuenta = "efectivo" | "banco" | "billetera_digital";

export interface CuentaDTO {
  id: string;
  nombre: string;
  tipo: TipoCuenta;
  moneda_codigo: string;
  saldo: string; // string para preservar precisión decimal
  creada_en: string; // ISO 8601
}