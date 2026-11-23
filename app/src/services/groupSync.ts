import { invoke } from "@tauri-apps/api/core";

export type GroupAccountSelection = {
  grupo_id: string;
  cuenta_id: string;
  moneda: string;
};

export type SyncStatus = {
  cursor: number;
  ultima_sincronizacion: string | null;
  sesion_vencida: boolean;
};

export function obtenerCuentaGrupo(grupoId: string) {
  return invoke<GroupAccountSelection | null>("obtener_cuenta_grupo", {
    grupo_id: grupoId,
  });
}

export function elegirCuentaGrupo(
  seleccion: GroupAccountSelection,
  monedaGrupo: string,
) {
  return invoke<number>("elegir_cuenta_grupo", {
    seleccion,
    moneda_grupo: monedaGrupo,
  });
}

export function sincronizarGrupos() {
  return invoke<number>("sincronizar_grupos");
}

export function obtenerEstadoSincronizacion() {
  return invoke<SyncStatus>("estado_sincronizacion");
}
