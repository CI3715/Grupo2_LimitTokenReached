//Este es el hook que invoca la info del endpoint registrar_tasa

import {invoke} from "@tauri-apps/api/core"; 
import type { TasaDTO } from "@/types/tasas";
import type { RegisterRateFormValues } from "@/types/schemas";

export type { TasaDTO } from "@/types/tasas";
export type { RegisterRateFormValues };

export function registrarTasa(
    datos: RegisterRateFormValues,
): Promise<TasaDTO>{
    return invoke<TasaDTO>("registrar_tasa", {datos});
};
