"use client";
import { useEffect, useState } from "react";
import { GroupAccountSelector, PendingGroupExpenses, QueuedGroupExpenses, SyncButton } from "@/components/group-sync/GroupSyncIndicators";
import { useGroupSync } from "@/features/group-sync/GroupSyncProvider";
import { elegirCuentaGrupo, obtenerCuentaGrupo } from "@/services/groupSync";
import { listAccounts } from "@/services/listAccounts";
import type { Account } from "@/types/accounts";
import type { GrupoDTO } from "@/types/social";

export function GroupSyncPanel({ grupo }: { grupo: GrupoDTO }) {
  const { estado, pendientes, cola, sincronizando, error, sync, actualizar } = useGroupSync();
  const [cuentas, setCuentas] = useState<Account[]>([]);
  const [seleccion, setSeleccion] = useState("");
  const [guardando, setGuardando] = useState(false);
  const [errorCuenta, setErrorCuenta] = useState("");
  useEffect(() => {
    let activo = true;
    void Promise.all([listAccounts(), obtenerCuentaGrupo(grupo.id)]).then(([lista, cuenta]) => {
      if (activo) { setCuentas(lista); setSeleccion(cuenta?.cuenta_id ?? ""); setErrorCuenta(""); }
    }).catch(() => { if (activo) setErrorCuenta("No se pudo cargar la cuenta del grupo."); });
    return () => { activo = false; };
  }, [grupo.id]);
  async function elegir(id: string) {
    if (!id) return;
    setGuardando(true); setErrorCuenta("");
    try {
      await elegirCuentaGrupo({ grupo_id: grupo.id, cuenta_id: id, moneda: grupo.moneda }, grupo.moneda);
      setSeleccion(id); await actualizar(); window.dispatchEvent(new Event("group-sync-updated"));
    } catch (cause) { setErrorCuenta(typeof cause === "string" ? cause : "No se pudo guardar la cuenta."); }
    finally { setGuardando(false); }
  }
  return <div style={{ display: "grid", gap: 16, marginBottom: 16 }}>
    <GroupAccountSelector cuentas={cuentas.map((c) => ({ id: c.id, nombre: c.nombre, moneda: c.moneda_codigo, activa: c.activa }))} moneda={grupo.moneda} seleccion={seleccion} guardando={guardando} onSelect={(id) => void elegir(id)} />
    {errorCuenta && <p role="alert">{errorCuenta}</p>}
    <SyncButton ultimaSincronizacion={estado?.ultima_sincronizacion ?? null} sincronizando={sincronizando} sesionVencida={Boolean(estado?.sesion_vencida)} onSync={() => void sync()} />
    {error && <p role="alert">{error}</p>}
    <PendingGroupExpenses items={pendientes.filter((p) => p.grupo_id === grupo.id).map((p) => ({ id: p.id_evento, grupo: grupo.nombre, concepto: p.concepto, fecha: p.fecha, monto: p.monto_parte, moneda: p.moneda }))} />
    <QueuedGroupExpenses items={cola.filter((g) => g.grupo_id === grupo.id && (g.estado === "pendiente" || g.estado === "rechazado")).map((g) => ({ id: g.idempotency_key, concepto: g.concepto, monto: g.monto, moneda: g.moneda || grupo.moneda, estado: g.estado as "pendiente" | "rechazado", motivo: g.motivo ?? undefined }))} />
  </div>;
}
