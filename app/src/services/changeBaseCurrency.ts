import { invoke } from "@tauri-apps/api/core";

export async function changeBaseCurrency(codigo: string): Promise<void> {
  await invoke("cambiar_moneda_base", {
    datos: { codigo },
  });
}
