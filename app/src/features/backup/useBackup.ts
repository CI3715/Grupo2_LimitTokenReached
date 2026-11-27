"use client";

import { useCallback, useState } from "react";
import {
  exportBackup,
  importBackup,
  syncBackupDestination,
} from "@/services/backup";
import type {
  BackupSyncDestination,
  DeviceSyncResponse,
  ExportBackupResponse,
  ImportBackupResponse,
} from "@/types/backup";

export function backupErrorMessage(error: unknown) {
  const message =
    typeof error === "string"
      ? error
      : error instanceof Error
        ? error.message
        : "";
  const normalized = message.toLocaleLowerCase("es");

  if (
    normalized.includes("frase de paso incorrecta") ||
    normalized.includes("fraseincorrectaoarchivodanado")
  ) {
    return "La frase de paso no es correcta o el archivo está dañado.";
  }
  if (
    normalized.includes("no se pudo llegar al repositorio o a la carpeta") ||
    normalized.includes("destinoinaccesibleosinconexion")
  ) {
    return "No se pudo acceder al destino. Comprueba la ruta y vuelve a intentarlo.";
  }
  if (
    normalized.includes("acceso denegado") ||
    normalized.includes("os error 5") ||
    normalized.includes("permission denied")
  ) {
    return "No se pudo escribir en esa ubicación. Comprueba que sea una carpeta y que tengas permiso para guardar allí.";
  }
  if (
    normalized.includes("credenciales") ||
    normalized.includes("credencialesgitinvalidas")
  ) {
    return "Git rechazó las credenciales guardadas. Actualízalas en el gestor de credenciales de Git de este dispositivo y vuelve a sincronizar.";
  }
  if (
    normalized.includes("versión de formato no soportada") ||
    normalized.includes("versionformatonosoportada")
  ) {
    return message;
  }
  if (
    normalized.includes("evento con identificador repetido") ||
    normalized.includes("eventoconflictocontenido")
  ) {
    return message;
  }
  return message || "No se pudo completar la operación. Inténtalo nuevamente.";
}

export function useBackup() {
  const [exportResult, setExportResult] =
    useState<ExportBackupResponse | null>(null);
  const [importResult, setImportResult] =
    useState<ImportBackupResponse | null>(null);
  const [syncResult, setSyncResult] = useState<DeviceSyncResponse | null>(null);
  const [exportError, setExportError] = useState("");
  const [importError, setImportError] = useState("");
  const [syncError, setSyncError] = useState("");
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [syncing, setSyncing] = useState(false);

  const exportar = useCallback(async (datos: {
    ruta_destino: string;
    frase_de_paso: string | null;
    cifrar: boolean;
  }) => {
    setExporting(true);
    setExportError("");
    setExportResult(null);
    try {
      const result = await exportBackup(datos);
      setExportResult(result);
      return true;
    } catch (error: unknown) {
      console.error("No se pudo exportar la copia de seguridad.", error);
      setExportError(backupErrorMessage(error));
      return false;
    } finally {
      setExporting(false);
    }
  }, []);

  const importar = useCallback(async (datos: {
    ruta_archivo: string;
    frase_de_paso: string;
  }) => {
    setImporting(true);
    setImportError("");
    setImportResult(null);
    try {
      const result = await importBackup(datos);
      setImportResult(result);
      return true;
    } catch (error: unknown) {
      console.error("No se pudo importar la copia de seguridad.", error);
      setImportError(backupErrorMessage(error));
      return false;
    } finally {
      setImporting(false);
    }
  }, []);

  const sincronizar = useCallback(async (
    destino: BackupSyncDestination,
    frase: string,
  ) => {
    setSyncing(true);
    setSyncError("");
    try {
      const result = await syncBackupDestination(destino, frase);
      setSyncResult(result);
      return true;
    } catch (error: unknown) {
      console.error("No se pudo sincronizar el destino de respaldo.", error);
      setSyncError(backupErrorMessage(error));
      return false;
    } finally {
      setSyncing(false);
    }
  }, []);

  return {
    exportResult,
    importResult,
    syncResult,
    exportError,
    importError,
    syncError,
    exporting,
    importing,
    syncing,
    exportar,
    importar,
    sincronizar,
  };
}
