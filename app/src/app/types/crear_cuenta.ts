import type { TipoCuenta } from "./cuenta";

export interface CrearCuentaRequest {
  nombre: string;
  tipo: TipoCuenta;
  moneda_codigo: string;
  saldo_inicial: string; // string para preservar decimales exactos
}