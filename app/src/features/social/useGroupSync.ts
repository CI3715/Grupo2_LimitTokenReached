"use client";

import { useCallback, useEffect, useState } from "react";

import type { Account } from "@/types/accounts";
import { listAccounts } from "@/services/listAccounts";
import {
  elegirCuentaGrupo,
  obtenerCuentaGrupo,
  obtenerEstadoSincronizacion,
  sincronizarGrupos,
  type SyncStatus,
} from "@/services/groupSync";

export function useGroupSync(grupoId: string | null, moneda: string) {
  const [cuentas, setCuentas] = useState<Account[]>([]);
  const [seleccion, setSeleccion] = useState("");
  const [estado, setEstado] = useState<SyncStatus>({
    cursor: 0,
    ultima_sincronizacion: null,
    sesion_vencida: false,
  });
  const [guardando, setGuardando] = useState(false);
  const [sincronizando, setSincronizando] = useState(false);
  const [error, setError] = useState("");

  const cargar = useCallback(async () => {
    if (!grupoId) return;
    setError("");
    try {
      const [cuentasDisponibles, seleccionGuardada, estadoActual] =
        await Promise.all([
          listAccounts(),
          obtenerCuentaGrupo(grupoId),
          obtenerEstadoSincronizacion(),
        ]);
      setCuentas(cuentasDisponibles);
      setSeleccion(seleccionGuardada?.cuenta_id ?? "");
      setEstado(estadoActual);
    } catch (cause: unknown) {
      console.error("No se pudo cargar la sincronización del grupo.", cause);
      setError(
        typeof cause === "string"
          ? cause
          : "No se pudo cargar la sincronización del grupo.",
      );
    }
  }, [grupoId]);

  useEffect(() => {
    void Promise.resolve().then(cargar);
  }, [cargar]);

  async function vincular(cuentaId: string) {
    if (!grupoId || !cuentaId) return false;
    setGuardando(true);
    setError("");
    try {
      await elegirCuentaGrupo(
        { grupo_id: grupoId, cuenta_id: cuentaId, moneda },
        moneda,
      );
      setSeleccion(cuentaId);
      return true;
    } catch (cause: unknown) {
      console.error("No se pudo vincular la cuenta al grupo.", cause);
      setError(
        typeof cause === "string"
          ? cause
          : "No se pudo vincular la cuenta al grupo.",
      );
      return false;
    } finally {
      setGuardando(false);
    }
  }

  async function sincronizar() {
    setSincronizando(true);
    setError("");
    try {
      await sincronizarGrupos();
      await cargar();
      return true;
    } catch (cause: unknown) {
      console.error("No se pudo sincronizar los gastos del grupo.", cause);
      setError(
        typeof cause === "string"
          ? cause
          : "No se pudo sincronizar los gastos del grupo.",
      );
      return false;
    } finally {
      setSincronizando(false);
    }
  }

  return {
    cuentas,
    seleccion,
    estado,
    guardando,
    sincronizando,
    error,
    vincular,
    sincronizar,
  };
}
