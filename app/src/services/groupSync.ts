import { invoke } from "@tauri-apps/api/core";

export interface PartePendiente {
  id_evento: string; grupo_id: string; gasto_id: string; concepto: string;
  fecha: string; moneda: string; monto_parte: string; incluye_redondeo: boolean; posicion_cursor: number;
}
export interface GastoCola {
  idempotency_key: string; grupo_id: string; concepto: string; monto: string;
  moneda: string; estado: string; motivo: string | null; orden: number;
}
export interface EstadoSync { cursor: number; ultima_sincronizacion: string | null; sesion_vencida: boolean }
export interface CuentaGrupo { grupo_id: string; cuenta_id: string; moneda: string }
export const sincronizarGrupos = () => invoke<number>("sincronizar_grupos");
export const estadoSincronizacion = () => invoke<EstadoSync>("estado_sincronizacion");
export const listarPendientesGrupo = () => invoke<PartePendiente[]>("listar_partes_grupo_pendientes");
export const listarColaGrupo = () => invoke<GastoCola[]>("listar_cola_gastos_grupo");
export const obtenerCuentaGrupo = (grupo_id: string) => invoke<CuentaGrupo | null>("obtener_cuenta_grupo", { grupo_id });
export const elegirCuentaGrupo = (seleccion: CuentaGrupo, moneda_grupo: string) => invoke<number>("elegir_cuenta_grupo", { seleccion, moneda_grupo });
