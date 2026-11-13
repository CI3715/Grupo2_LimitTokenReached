"use client";

import { useCallback, useEffect, useState } from "react";

import {
  crearGrupo,
  invitarMiembroGrupo,
  listarGrupos,
  obtenerDetalleGrupo,
  previsualizarDivisionGrupo,
  registrarGastoGrupo,
  responderInvitacionGrupo,
} from "@/services/social";
import type {
  CrearGrupoRequestDTO,
  DetalleGrupoDTO,
  ListaGruposDTO,
  ParteGastoDTO,
  RegistrarGastoRequestDTO,
} from "@/types/social";

const listaInicial: ListaGruposDTO = { miembros: [], invitaciones: [] };

function mensajeDeError(error: unknown, fallback: string) {
  return typeof error === "string" ? error : fallback;
}

export function useGroups(activo: boolean) {
  const [grupos, setGrupos] = useState<ListaGruposDTO>(listaInicial);
  const [detalle, setDetalle] = useState<DetalleGrupoDTO | null>(null);
  const [grupoSeleccionado, setGrupoSeleccionado] = useState("");
  const [cargando, setCargando] = useState(false);
  const [cargandoDetalle, setCargandoDetalle] = useState(false);
  const [guardando, setGuardando] = useState(false);
  const [error, setError] = useState("");

  const cargarGrupos = useCallback(async () => {
    if (!activo) return;
    setCargando(true);
    setError("");
    try {
      setGrupos(await listarGrupos());
    } catch (cause: unknown) {
      console.error("No se pudieron cargar los grupos.", cause);
      setError(mensajeDeError(cause, "No se pudieron cargar los grupos."));
    } finally {
      setCargando(false);
    }
  }, [activo]);

  const cargarDetalle = useCallback(async (grupoId: string) => {
    setGrupoSeleccionado(grupoId);
    setDetalle(null);
    setCargandoDetalle(true);
    setError("");
    try {
      setDetalle(await obtenerDetalleGrupo(grupoId));
    } catch (cause: unknown) {
      console.error("No se pudo cargar el grupo.", cause);
      setError(mensajeDeError(cause, "No se pudo cargar el grupo."));
    } finally {
      setCargandoDetalle(false);
    }
  }, []);

  const previsualizar = useCallback(
    (datos: RegistrarGastoRequestDTO): Promise<ParteGastoDTO[]> =>
      previsualizarDivisionGrupo(datos),
    [],
  );

  useEffect(() => {
    void Promise.resolve().then(cargarGrupos);
  }, [cargarGrupos]);

  async function crear(datos: CrearGrupoRequestDTO) {
    setGuardando(true);
    setError("");
    try {
      const creado = await crearGrupo(datos);
      await cargarGrupos();
      await cargarDetalle(creado.id);
      return true;
    } catch (cause: unknown) {
      setError(mensajeDeError(cause, "No se pudo crear el grupo."));
      return false;
    } finally {
      setGuardando(false);
    }
  }

  async function responderInvitacion(grupoId: string, aceptar: boolean) {
    setGuardando(true);
    setError("");
    try {
      await responderInvitacionGrupo(grupoId, aceptar);
      await cargarGrupos();
      if (aceptar) await cargarDetalle(grupoId);
    } catch (cause: unknown) {
      setError(mensajeDeError(cause, "No se pudo responder la invitación."));
    } finally {
      setGuardando(false);
    }
  }

  async function invitar(grupoId: string, nombreUsuario: string) {
    setGuardando(true);
    setError("");
    try {
      await invitarMiembroGrupo(grupoId, { usuario: nombreUsuario });
      await cargarGrupos();
      await cargarDetalle(grupoId);
      return true;
    } catch (cause: unknown) {
      setError(mensajeDeError(cause, "No se pudo invitar al miembro."));
      return false;
    } finally {
      setGuardando(false);
    }
  }

  async function registrarGasto(
    grupoId: string,
    datos: RegistrarGastoRequestDTO,
  ) {
    setGuardando(true);
    setError("");
    try {
      await registrarGastoGrupo(grupoId, datos);
      await cargarDetalle(grupoId);
      return true;
    } catch (cause: unknown) {
      setError(mensajeDeError(cause, "No se pudo registrar el gasto."));
      return false;
    } finally {
      setGuardando(false);
    }
  }

  return {
    grupos,
    detalle,
    grupoSeleccionado,
    cargando,
    cargandoDetalle,
    guardando,
    error,
    cargarGrupos,
    cargarDetalle,
    previsualizar,
    crear,
    responderInvitacion,
    invitar,
    registrarGasto,
  };
}
