import { invoke } from "@tauri-apps/api/core";
import type {
  ConsultarPatrimonioRequest,
  Patrimonio,
} from "@/types/patrimonio";

export function consultarPatrimonio(
  request: ConsultarPatrimonioRequest,
): Promise<Patrimonio> {
  return invoke<Patrimonio>("consultar_patrimonio", { datos: request });
}
