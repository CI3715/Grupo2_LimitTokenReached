import type { CategoriaDTO } from "./movements";

// Debe coincidir con el catálogo fijo de crates/dominio/src/categoria.rs.
export const movementCategories: CategoriaDTO[] = [
  { id: "00000000-0000-0000-0000-000000000001", nombre: "Alimentación", tipo: "Gasto" },
  { id: "00000000-0000-0000-0000-000000000002", nombre: "Transporte", tipo: "Gasto" },
  { id: "00000000-0000-0000-0000-000000000003", nombre: "Servicios", tipo: "Gasto" },
  { id: "00000000-0000-0000-0000-000000000004", nombre: "Salud", tipo: "Gasto" },
  { id: "00000000-0000-0000-0000-000000000005", nombre: "Educación", tipo: "Gasto" },
  { id: "00000000-0000-0000-0000-000000000006", nombre: "Entretenimiento", tipo: "Gasto" },
  { id: "00000000-0000-0000-0000-000000000007", nombre: "Otros gastos", tipo: "Gasto" },
  { id: "00000000-0000-0000-0000-000000000008", nombre: "Salario", tipo: "Ingreso" },
  { id: "00000000-0000-0000-0000-000000000009", nombre: "Ventas", tipo: "Ingreso" },
  { id: "00000000-0000-0000-0000-00000000000a", nombre: "Regalos", tipo: "Ingreso" },
  { id: "00000000-0000-0000-0000-00000000000b", nombre: "Otros ingresos", tipo: "Ingreso" },
];
