import { invoke } from "@tauri-apps/api/core";
import type {
  DeviceSyncResponse,
  ExportBackupResponse,
  ImportBackupResponse,
} from "@/types/backup";

export function exportBackup(datos: {
  ruta_destino: string;
  frase_de_paso: string | null;
  cifrar: boolean;
}) {
  return invoke<ExportBackupResponse>("exportar_libro", { datos });
}

export function importBackup(datos: {
  ruta_archivo: string;
  frase_de_paso: string;
}) {
  return invoke<ImportBackupResponse>("importar_libro", { datos });
}

export function syncSharedFolder(
  ruta_carpeta: string,
  frase_de_paso: string,
) {
  return invoke<DeviceSyncResponse>("sincronizar_carpeta_libro", {
    rutaCarpeta: ruta_carpeta,
    fraseDePaso: frase_de_paso,
  });
}
