import { invoke } from "@tauri-apps/api/core";
import type { CategoriaDTO } from "@/types/movements";
import type { CrearCategoriaRequest } from "@/types/categories";

export async function createCategory(
  request: CrearCategoriaRequest,
): Promise<CategoriaDTO> {
  return invoke<CategoriaDTO>("crear_categoria", { datos: request });
}
