import { invoke } from "@tauri-apps/api/core";
import type {
  BackupSyncDestination,
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

export function syncGitRepository(
  configuracion: Extract<BackupSyncDestination, { tipo: "git" }>,
  frase_de_paso: string,
) {
  return invoke<DeviceSyncResponse>("sincronizar_git_libro", {
    configuracion: {
      ...configuracion,
      credenciales: null,
    },
    fraseDePaso: frase_de_paso,
  });
}

export function syncBackupDestination(
  destino: BackupSyncDestination,
  frase_de_paso: string,
) {
  return destino.tipo === "git"
    ? syncGitRepository(destino, frase_de_paso)
    : syncSharedFolder(destino.ruta_o_url, frase_de_paso);
}
