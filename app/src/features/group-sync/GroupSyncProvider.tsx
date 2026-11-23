"use client";
import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { useAuthSession } from "@/features/auth/useAuthSession";
import { startSyncSchedule } from "@/features/group-sync/schedule";
import { estadoSincronizacion, listarColaGrupo, listarPendientesGrupo, sincronizarGrupos, type EstadoSync, type GastoCola, type PartePendiente } from "@/services/groupSync";

type SyncContext = { estado: EstadoSync | null; pendientes: PartePendiente[]; cola: GastoCola[]; sincronizando: boolean; error: string; sync: () => Promise<void>; actualizar: () => Promise<void> };
const Context = createContext<SyncContext | null>(null);
export function GroupSyncProvider({ children }: { children: ReactNode }) {
  const { usuario, esAplicacionTauri } = useAuthSession();
  const [estado, setEstado] = useState<EstadoSync | null>(null);
  const [pendientes, setPendientes] = useState<PartePendiente[]>([]);
  const [cola, setCola] = useState<GastoCola[]>([]);
  const [sincronizando, setSincronizando] = useState(false);
  const [error, setError] = useState("");
  const ocupado = useRef(false);
  const vencida = useRef(false);
  const actualizar = useCallback(async () => {
    const [nuevoEstado, nuevasPartes, nuevaCola] = await Promise.all([estadoSincronizacion(), listarPendientesGrupo(), listarColaGrupo()]);
    setEstado(nuevoEstado); setPendientes(nuevasPartes); setCola(nuevaCola);
  }, []);
  const sync = useCallback(async () => {
    if (ocupado.current || vencida.current) return;
    ocupado.current = true; setSincronizando(true); setError("");
    try { await sincronizarGrupos(); window.dispatchEvent(new Event("group-sync-updated")); }
    catch (cause) {
      const mensaje = typeof cause === "string" ? cause : "No se pudo sincronizar.";
      setError(mensaje);
      if (mensaje.includes("Tu sesión venció")) { vencida.current = true; window.dispatchEvent(new Event("auth-session-changed")); }
    } finally {
      try { await actualizar(); } catch { setError("No se pudo consultar el estado de sincronización."); }
      ocupado.current = false; setSincronizando(false);
    }
  }, [actualizar]);
  const usuarioId = usuario?.nombre_usuario;
  useEffect(() => {
    if (!esAplicacionTauri || !usuarioId) return;
    vencida.current = false;
    const programador = startSyncSchedule({ sync, onError: () => setError("No se pudo sincronizar.") });
    return () => programador.stop();
  }, [esAplicacionTauri, usuarioId, sync]);
  return <Context.Provider value={{ estado, pendientes, cola, sincronizando, error, sync, actualizar }}>{children}</Context.Provider>;
}
export function useGroupSync() { const value = useContext(Context); if (!value) throw new Error("Falta GroupSyncProvider"); return value; }
