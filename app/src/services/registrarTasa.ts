//Este es el hook que invoca la info del endpoint registrar_tasa

import {invoke} from "@tauri-apps/api/core"; 
import type { RegistrarTasaRequest, TasaDTO } from "@/types/tasas";

export type { RegistrarTasaRequest, TasaDTO } from "@/types/tasas";

export function registrarTasa(
    datos:RegistrarTasaRequest, 
): Promise<TasaDTO>{
    return invoke<TasaDTO>("registrar_tasa", {datos});
};
