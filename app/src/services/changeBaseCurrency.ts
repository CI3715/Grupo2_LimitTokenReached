import { invoke } from "@tauri-apps/api/core";

export function changeBaseCurrency(codigo: string): Promise<void> {
  return invoke("cambiar_moneda_base", {
    datos: { codigo },
  });
}
