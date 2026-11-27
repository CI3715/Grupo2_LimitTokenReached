import { z } from "zod";

export const exportBackupSchema = z
  .object({
    ruta_destino: z.string().trim().min(1, "Indica la ruta de destino."),
    cifrar: z.boolean(),
    frase_de_paso: z.string(),
    confirmar_frase: z.string(),
  })
  .superRefine((values, context) => {
    if (!values.cifrar) return;

    if (!values.frase_de_paso.trim()) {
      context.addIssue({
        code: "custom",
        path: ["frase_de_paso"],
        message: "La frase de paso es obligatoria.",
      });
    }

    if (values.frase_de_paso !== values.confirmar_frase) {
      context.addIssue({
        code: "custom",
        path: ["confirmar_frase"],
        message: "Las frases de paso no coinciden.",
      });
    }
  });

export const importBackupSchema = z.object({
  ruta_archivo: z.string().trim().min(1, "Selecciona o indica el archivo."),
  frase_de_paso: z.string(),
});

export const deviceSyncSchema = z
  .object({
    tipo: z.enum(["carpeta", "git"]),
    ruta_o_url: z.string().trim().min(1, "Indica la ruta o URL de destino."),
    usuario_git: z.string().trim(),
    credenciales_git: z.string(),
    frase_de_paso: z.string().min(1, "Escribe la frase de paso."),
  })
  .superRefine((values, context) => {
    if (
      values.tipo === "git" &&
      !values.credenciales_git.trim()
    ) {
      context.addIssue({
        code: "custom",
        path: ["credenciales_git"],
        message: "Indica un token o contraseña para el repositorio.",
      });
    }
  });

export type ExportBackupFormValues = z.infer<typeof exportBackupSchema>;
export type ImportBackupFormValues = z.infer<typeof importBackupSchema>;
export type DeviceSyncFormValues = z.infer<typeof deviceSyncSchema>;
