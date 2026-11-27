export type RejectedBackupEvent = {
  id_evento: string;
  motivo: string;
};

export type ImportBackupReport = {
  eventos_nuevos: number;
  eventos_ya_existentes: number;
  eventos_rechazados: RejectedBackupEvent[];
  fecha_ultima_sincronizacion: string;
};

export type ImportBackupResponse = {
  exitoso: boolean;
  reporte: ImportBackupReport;
};

export type ExportBackupResponse = {
  exitoso: boolean;
  ruta_archivo: string;
  cantidad_eventos: number;
};

export type DeviceSyncResponse = {
  exitoso: boolean;
  eventos_enviados: number;
  eventos_recibidos: number;
  eventos_rechazados: RejectedBackupEvent[];
  fecha_ultima_sync: string;
};

export type BackupSyncDestination =
  | { tipo: "carpeta"; ruta_o_url: string }
  | { tipo: "git"; ruta_o_url: string };
