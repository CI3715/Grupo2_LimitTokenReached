import { invoke } from "@tauri-apps/api/core";
import type { CategoriaDTO } from "@/types/movements";

export async function listCategories(): Promise<CategoriaDTO[]> {
  return invoke<CategoriaDTO[]>("listar_categorias");
}
