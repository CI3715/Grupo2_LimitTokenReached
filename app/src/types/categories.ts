export type TipoCategoria = "Gasto" | "Ingreso";

export interface CategoriaDTO {
  id: string;
  nombre: string;
  tipo: TipoCategoria;
}

export interface CrearCategoriaRequest {
  nombre: string;
  tipo: TipoCategoria;
}
