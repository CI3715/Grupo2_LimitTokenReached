import { invoke } from "@tauri-apps/api/core";
import type { CategoriaDTO } from "@/types/movements";
import type { CreateCategoryFormValues } from "@/types/schemas";

export async function createCategory(
  request: CreateCategoryFormValues,
): Promise<CategoriaDTO> {
  return invoke<CategoriaDTO>("crear_categoria", { datos: request });
}
