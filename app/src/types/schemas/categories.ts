import { z } from "zod";
import { formFields } from "./fields";
import { movementTypeSchema } from "./movements";

export const createCategorySchema = z.object({
  nombre: formFields.nombre,
  tipo: movementTypeSchema,
});

export type CreateCategoryFormValues = z.infer<typeof createCategorySchema>;
