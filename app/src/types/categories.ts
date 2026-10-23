import type { TipoRegistroMovimiento } from "./movements";

export interface CrearCategoriaRequest {
  nombre: string;
  tipo: TipoRegistroMovimiento;
}
