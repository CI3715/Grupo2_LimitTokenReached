"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useCallback, useEffect, useRef, useState } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { FormField } from "@/components/forms/FormField";
import type {
  BackupSyncDestination,
  DeviceSyncResponse,
  ExportBackupResponse,
  ImportBackupResponse,
} from "@/types/backup";
import {
  deviceSyncSchema,
  exportBackupSchema,
  importBackupSchema,
  type DeviceSyncFormValues,
  type ExportBackupFormValues,
  type ImportBackupFormValues,
} from "@/types/schemas";

const DESTINATION_STORAGE_KEY = "cuentas-claras.backup-destination";

function filePath(file: File | undefined) {
  if (!file || !("path" in file)) return "";
  const path = file.path;
  return typeof path === "string" ? path : "";
}

function formattedDate(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat("es-VE", {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(date);
}

function exportDestinationPath(destination: string) {
  const path = destination.trim();
  if (/\.ccbak$/i.test(path)) return path;

  const directory = path.replace(/[\\/]+$/, "");
  const separator = path.includes("\\") ? "\\" : "/";
  return `${directory}${separator}respaldo.ccbak`;
}

type BackupActions = {
  exportResult: ExportBackupResponse | null;
  importResult: ImportBackupResponse | null;
  syncResult: DeviceSyncResponse | null;
  exportError: string;
  importError: string;
  syncError: string;
  exporting: boolean;
  importing: boolean;
  syncing: boolean;
  exportar: (datos: {
    ruta_destino: string;
    frase_de_paso: string | null;
    cifrar: boolean;
  }) => Promise<boolean>;
  importar: (datos: {
    ruta_archivo: string;
    frase_de_paso: string;
  }) => Promise<boolean>;
  sincronizar: (
    destino: BackupSyncDestination,
    frase: string,
  ) => Promise<boolean>;
};

export function BackupWorkspace({ backup }: { backup: BackupActions }) {
  const importPicker = useRef<HTMLInputElement>(null);
  const folderPicker = useRef<HTMLInputElement>(null);
  const setFolderPicker = useCallback((element: HTMLInputElement | null) => {
    folderPicker.current = element;
    element?.setAttribute("webkitdirectory", "");
  }, []);
  const syncForm = useForm<DeviceSyncFormValues>({
    resolver: zodResolver(deviceSyncSchema),
    defaultValues: {
      tipo: "carpeta",
      ruta_o_url: "",
      frase_de_paso: "",
    },
  });
  const selectedMode = useWatch({ control: syncForm.control, name: "tipo" });
  const syncDestination = useWatch({
    control: syncForm.control,
    name: "ruta_o_url",
  });
  const syncPassphrase = useWatch({
    control: syncForm.control,
    name: "frase_de_paso",
  });
  const [pickerNotice, setPickerNotice] = useState("");
  const [syncNotice, setSyncNotice] = useState("");

  const exportForm = useForm<ExportBackupFormValues>({
    resolver: zodResolver(exportBackupSchema),
    defaultValues: {
      ruta_destino: "",
      cifrar: true,
      frase_de_paso: "",
      confirmar_frase: "",
    },
  });
  const encryptionEnabled = useWatch({
    control: exportForm.control,
    name: "cifrar",
  });
  const importForm = useForm<ImportBackupFormValues>({
    resolver: zodResolver(importBackupSchema),
    defaultValues: { ruta_archivo: "", frase_de_paso: "" },
  });

  const { sincronizar } = backup;
  const sync = useCallback(
    async (destino: BackupSyncDestination, frase: string) => {
      const successful = await sincronizar(destino, frase);
      if (successful) setSyncNotice("Sincronización completada.");
      else setSyncNotice("");
      return successful;
    },
    [sincronizar],
  );

  useEffect(() => {
    void Promise.resolve().then(() => {
      try {
        const stored = window.localStorage.getItem(DESTINATION_STORAGE_KEY);
        if (stored) {
          const config: unknown = JSON.parse(stored);
          if (
            typeof config === "object" &&
            config !== null &&
            "tipo" in config &&
            (config.tipo === "carpeta" || config.tipo === "git") &&
            "ruta_o_url" in config &&
            typeof config.ruta_o_url === "string"
          ) {
            syncForm.setValue("tipo", config.tipo);
            syncForm.setValue("ruta_o_url", config.ruta_o_url);
          }
        }
      } catch (error: unknown) {
        console.error("No se pudo leer el destino de sincronización guardado.", error);
        setSyncNotice("No se pudo cargar el destino guardado. Configúralo de nuevo.");
      }
    });
  }, [syncForm]);

  function choosePath(
    file: File | undefined,
    setValue: (path: string) => void,
    fileName: string,
  ) {
    const path = filePath(file);
    if (path) {
      setValue(path);
      setPickerNotice("");
    } else if (file) {
      setPickerNotice(
        `Se seleccionó “${file.name}”, pero el selector no entrega su ruta completa. Escríbela en el campo para continuar.`,
      );
    }
    const input = document.getElementById(fileName);
    if (input instanceof HTMLInputElement) input.value = "";
  }

  async function submitExport(values: ExportBackupFormValues) {
    const success = await backup.exportar({
      ruta_destino: exportDestinationPath(values.ruta_destino),
      cifrar: values.cifrar,
      frase_de_paso: values.cifrar ? values.frase_de_paso : null,
    });
    if (success) {
      exportForm.reset({
        ruta_destino: values.ruta_destino,
        cifrar: values.cifrar,
        frase_de_paso: "",
        confirmar_frase: "",
      });
    }
  }

  async function submitImport(values: ImportBackupFormValues) {
    const success = await backup.importar(values);
    if (success) {
      importForm.reset({ ruta_archivo: values.ruta_archivo, frase_de_paso: "" });
    }
  }

  async function submitSync(values: DeviceSyncFormValues) {
    setSyncNotice("");
    const config: BackupSyncDestination = {
      tipo: values.tipo,
      ruta_o_url: values.ruta_o_url,
    };
    try {
      window.localStorage.setItem(
        DESTINATION_STORAGE_KEY,
        JSON.stringify(config),
      );
    } catch (error: unknown) {
      console.error("No se pudo guardar el destino de sincronización.", error);
      setSyncNotice("No se pudo guardar el destino en este dispositivo.");
      return;
    }

    await sync(config, values.frase_de_paso);
  }

  return (
    <div className="backup-sections">
      <section className="backup-panel" aria-labelledby="backup-export-title">
        <div className="backup-panel-heading">
          <span className="backup-icon" aria-hidden="true">↑</span>
          <div>
            <h2 id="backup-export-title">Exportar copia de seguridad</h2>
            <p>Crea una copia local de tus eventos.</p>
          </div>
        </div>
        <FormProvider {...exportForm}>
          <form
            className="backup-form backup-export-form form-stack"
            onSubmit={exportForm.handleSubmit(submitExport)}
          >
            <FormField<ExportBackupFormValues>
              name="ruta_destino"
              label="Ruta de destino"
              help="Escribe una carpeta para guardar respaldo.ccbak, o indica directamente la ruta de un archivo .ccbak."
              className="backup-export-path"
            >
              {(props) => (
                <input
                  {...exportForm.register("ruta_destino")}
                  {...props}
                  autoComplete="off"
                  placeholder={"C:\\User\\Desktop\\Copia\\"}
                />
              )}
            </FormField>
            <div className="backup-encryption-control">
              <strong>Cifrar copia de seguridad</strong>
              <div className="backup-toggle-group" role="group" aria-label="Cifrado de la copia">
                <span
                  className={`backup-toggle-indicator${!encryptionEnabled ? " is-unciphered" : ""}`}
                  aria-hidden="true"
                />
                <button
                  type="button"
                  aria-pressed={encryptionEnabled}
                  className={encryptionEnabled ? "is-selected" : undefined}
                  onClick={() =>
                    exportForm.setValue("cifrar", true, {
                      shouldDirty: true,
                      shouldValidate: true,
                    })
                  }
                >
                  Cifrar
                </button>
                <button
                  type="button"
                  aria-pressed={!encryptionEnabled}
                  className={!encryptionEnabled ? "is-selected" : undefined}
                  onClick={() =>
                    exportForm.setValue("cifrar", false, {
                      shouldDirty: true,
                      shouldValidate: true,
                    })
                  }
                >
                  No cifrar
                </button>
              </div>
            </div>
            {!encryptionEnabled && (
              <p className="backup-warning" role="alert">
                La copia quedará sin cifrar y contendrá información financiera legible.
              </p>
            )}
            {encryptionEnabled && (
              <div className="form-grid form-grid-two-columns backup-passphrase-fields">
              <FormField<ExportBackupFormValues>
                name="frase_de_paso"
                label="Frase de paso"
              >
                {(props) => (
                  <input
                    {...exportForm.register("frase_de_paso")}
                    {...props}
                    type="password"
                    autoComplete="new-password"
                  />
                )}
              </FormField>
              <FormField<ExportBackupFormValues>
                name="confirmar_frase"
                label="Repite la frase de paso"
              >
                {(props) => (
                  <input
                    {...exportForm.register("confirmar_frase")}
                    {...props}
                    type="password"
                    autoComplete="new-password"
                  />
                )}
              </FormField>
              </div>
            )}
            {backup.exportError && (
              <p className="backup-error" role="alert">{backup.exportError}</p>
            )}
            {backup.exportResult && (
              <p className="backup-success" role="status">
                Copia creada con {backup.exportResult.cantidad_eventos} eventos en{" "}
                {backup.exportResult.ruta_archivo}.
              </p>
            )}
            <button
              className="account-primary"
              type="submit"
              disabled={backup.exporting}
            >
              {backup.exporting ? "Exportando…" : "Exportar copia"}
            </button>
          </form>
        </FormProvider>
      </section>

      <section className="backup-panel" aria-labelledby="backup-import-title">
        <div className="backup-panel-heading">
          <span className="backup-icon" aria-hidden="true">↓</span>
          <div>
            <h2 id="backup-import-title">Importar copia de seguridad</h2>
            <p>Fusiona los eventos de una copia con este dispositivo.</p>
          </div>
        </div>
        <FormProvider {...importForm}>
          <form
            className="backup-form backup-import-form form-stack"
            onSubmit={importForm.handleSubmit(submitImport)}
          >
          <FormField<ImportBackupFormValues>
            name="ruta_archivo"
            label="Archivo de respaldo"
            help="Selecciona un archivo .ccbak o escribe su ruta completa."
            className="backup-import-path"
          >
            {(props) => (
              <div className="backup-path-control">
                <input
                  {...importForm.register("ruta_archivo")}
                  {...props}
                  autoComplete="off"
                  placeholder={"C:\\Copias\\respaldo.ccbak"}
                />
                <input
                  ref={importPicker}
                  className="backup-file-picker"
                  type="file"
                  accept=".ccbak"
                  aria-label="Seleccionar archivo de respaldo"
                  onChange={(event) =>
                    choosePath(
                      event.currentTarget.files?.[0],
                      (path) => importForm.setValue("ruta_archivo", path, { shouldValidate: true }),
                      "backup-import-picker",
                    )
                  }
                  id="backup-import-picker"
                />
                <button
                  type="button"
                  className="account-secondary"
                  onClick={() => importPicker.current?.click()}
                >
                  Examinar
                </button>
              </div>
            )}
          </FormField>
          <FormField<ImportBackupFormValues>
            name="frase_de_paso"
            label="Frase de paso"
            className="backup-import-passphrase"
          >
            {(props) => (
              <input
                {...importForm.register("frase_de_paso")}
                {...props}
                type="password"
                autoComplete="current-password"
              />
            )}
          </FormField>
          {pickerNotice && <p className="backup-note" role="status">{pickerNotice}</p>}
          {backup.importError && (
            <p className="backup-error" role="alert">{backup.importError}</p>
          )}
          <button className="account-primary" type="submit" disabled={backup.importing}>
            {backup.importing ? "Importando…" : "Importar copia"}
          </button>
          </form>
        </FormProvider>
        {backup.importResult && (
          <div className="backup-report" role="status">
            <h3>Resumen de importación</h3>
            <dl className="backup-counts">
              <div><dt>Eventos nuevos</dt><dd>{backup.importResult.reporte.eventos_nuevos}</dd></div>
              <div><dt>Ya existían</dt><dd>{backup.importResult.reporte.eventos_ya_existentes}</dd></div>
              <div><dt>Rechazados</dt><dd>{backup.importResult.reporte.eventos_rechazados.length}</dd></div>
            </dl>
            {backup.importResult.reporte.eventos_rechazados.length > 0 && (
              <div>
                <h4>Eventos rechazados y motivo</h4>
                <ul className="backup-rejected">
                  {backup.importResult.reporte.eventos_rechazados.map((event) => (
                    <li key={event.id_evento}>
                      <strong>{event.id_evento}</strong>
                      <span>{event.motivo}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}
      </section>

      <section className="backup-panel" aria-labelledby="backup-sync-title">
        <div className="backup-panel-heading">
          <span className="backup-icon" aria-hidden="true">⇄</span>
          <div>
            <h2 id="backup-sync-title">Sincronización de dispositivos</h2>
            <p>Comparte cambios entre dispositivos mediante un destino común.</p>
          </div>
        </div>
        <FormProvider {...syncForm}>
          <form
            className="backup-form form-stack"
            onSubmit={syncForm.handleSubmit(submitSync)}
          >
          <fieldset className="backup-mode-selector">
            <legend>Modo de sincronización</legend>
            <label>
              <input
                type="radio"
                value="carpeta"
                {...syncForm.register("tipo")}
              />
              Carpeta compartida
            </label>
            <label>
              <input type="radio" value="git" {...syncForm.register("tipo")} />
              Repositorio Git
            </label>
          </fieldset>
          <FormField<DeviceSyncFormValues>
            name="ruta_o_url"
            label={selectedMode === "git" ? "URL del repositorio" : "Ruta de la carpeta"}
            help={
              selectedMode === "git"
                ? "Por ejemplo, la URL HTTPS del repositorio remoto."
                : "Elige la carpeta compartida entre tus dispositivos."
            }
          >
            {(props) => (
              <div className="backup-path-control">
                <input
                  {...syncForm.register("ruta_o_url")}
                  {...props}
                  autoComplete="off"
                  placeholder={selectedMode === "git" ? "https://…" : "C:\\Datos\\Compartida"}
                />
                {selectedMode === "carpeta" && (
                  <>
                    <input
                      ref={setFolderPicker}
                      className="backup-file-picker"
                      type="file"
                      aria-label="Elegir carpeta compartida"
                      onChange={(event) => {
                        const selected = event.currentTarget.files?.[0];
                        choosePath(
                          selected,
                          (path) => syncForm.setValue("ruta_o_url", path, { shouldValidate: true }),
                          "backup-folder-picker",
                        );
                      }}
                      id="backup-folder-picker"
                    />
                    <button
                      type="button"
                      className="account-secondary"
                      onClick={() => folderPicker.current?.click()}
                    >
                      Examinar
                    </button>
                  </>
                )}
              </div>
            )}
          </FormField>
          {selectedMode === "git" && (
            <p className="backup-note" role="status">
              Git usará el gestor de credenciales configurado en este dispositivo.
              Las credenciales no se guardan ni se envían desde la aplicación.
            </p>
          )}
          <FormField<DeviceSyncFormValues>
            name="frase_de_paso"
            label="Frase de paso de sincronización"
            help="No se guarda en el dispositivo; se mantiene en esta página para permitir nuevas sincronizaciones."
          >
            {(props) => (
              <input
                {...syncForm.register("frase_de_paso")}
                {...props}
                type="password"
                autoComplete="new-password"
              />
            )}
          </FormField>
          {pickerNotice && <p className="backup-note" role="status">{pickerNotice}</p>}
          {syncNotice && <p className="backup-note" role="status">{syncNotice}</p>}
          {backup.syncError && (
            <div className="backup-error-panel" role="alert">
              <p>{backup.syncError}</p>
              {backup.syncError.includes("credenciales") && selectedMode === "git" && (
                <p>
                  Actualiza las credenciales de este repositorio en el gestor de
                  credenciales de Git y pulsa “Sincronizar ahora” para reintentar.
                  La frase de paso se conserva para el reintento.
                </p>
              )}
            </div>
          )}
          <button
            className="account-primary"
            type="submit"
            disabled={backup.syncing}
          >
            {backup.syncing ? "Sincronizando…" : "Guardar destino y sincronizar"}
          </button>
          </form>
        </FormProvider>
        <div className="backup-sync-status">
          <h3>Estado de sincronización</h3>
          <p>
            La sincronización sólo se ejecuta cuando pulsas “Guardar destino y
            sincronizar” o “Sincronizar ahora”.
          </p>
          <p>
            Última sincronización exitosa:{" "}
            {backup.syncResult
              ? formattedDate(backup.syncResult.fecha_ultima_sync)
              : "sin registros en esta sesión"}
          </p>
          {backup.syncResult && (
            <div className="backup-report">
              <h4>Reporte de esta sincronización</h4>
              <dl className="backup-counts">
                <div><dt>Enviados</dt><dd>{backup.syncResult.eventos_enviados}</dd></div>
                <div><dt>Recibidos</dt><dd>{backup.syncResult.eventos_recibidos}</dd></div>
                <div><dt>Rechazados</dt><dd>{backup.syncResult.eventos_rechazados.length}</dd></div>
              </dl>
              {backup.syncResult.eventos_rechazados.length > 0 && (
                <ul className="backup-rejected">
                  {backup.syncResult.eventos_rechazados.map((event) => (
                    <li key={event.id_evento}>
                      <strong>{event.id_evento}</strong>
                      <span>{event.motivo}</span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
          <button
            type="button"
            className="account-secondary"
            disabled={
              backup.syncing ||
              !syncDestination.trim() ||
              !syncPassphrase.trim()
            }
            onClick={() =>
              void sync(
                { tipo: selectedMode, ruta_o_url: syncDestination.trim() },
                syncPassphrase,
              )
            }
          >
            {backup.syncing ? "Sincronizando…" : "Sincronizar ahora"}
          </button>
        </div>
      </section>
    </div>
  );
}
