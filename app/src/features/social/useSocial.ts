"use client";

import { useCallback, useEffect, useState } from "react";

import {
  buscarUsuario,
  eliminarAmistad,
  invitarAmistad,
  listarAmistades,
  responderInvitacionAmistad,
} from "@/services/social";
import type {
  InvitacionAmistadDTO,
  ListaAmistadesDTO,
  UsuarioResumenDTO,
} from "@/types/social";

const listaInicial: ListaAmistadesDTO = {
  amigos: [],
  enviadas: [],
  recibidas: [],
};

function mensajeDeError(error: unknown, fallback: string) {
  return typeof error === "string" ? error : fallback;
}

export function useSocial(activo: boolean) {
  const [amistades, setAmistades] = useState<ListaAmistadesDTO>(listaInicial);
  const [usuarioEncontrado, setUsuarioEncontrado] =
    useState<UsuarioResumenDTO | null>(null);
  const [cargando, setCargando] = useState(false);
  const [error, setError] = useState("");

  const cargarAmistades = useCallback(async () => {
    if (!activo) return;
    setCargando(true);
    setError("");
    try {
      setAmistades(await listarAmistades());
    } catch (cause: unknown) {
      console.error("No se pudieron cargar las amistades.", cause);
      setError(
        mensajeDeError(cause, "No se pudieron cargar las amistades."),
      );
    } finally {
      setCargando(false);
    }
  }, [activo]);

  useEffect(() => {
    void Promise.resolve().then(cargarAmistades);
  }, [cargarAmistades]);

  async function buscar(nombreUsuario: string) {
    setError("");
    setUsuarioEncontrado(null);
    try {
      const resultado = await buscarUsuario(nombreUsuario.trim());
      setUsuarioEncontrado(resultado);
      return resultado;
    } catch (cause: unknown) {
      const mensaje = mensajeDeError(
        cause,
        "No se encontró el usuario.",
      );
      setError(mensaje);
    }
  }

  async function invitar(nombreUsuario: string) {
    setError("");
    try {
      await invitarAmistad({ usuario: nombreUsuario.trim() });
      await cargarAmistades();
      setUsuarioEncontrado(null);
      return true;
    } catch (cause: unknown) {
      const mensaje = mensajeDeError(
        cause,
        "No se pudo enviar la invitación.",
      );
      setError(mensaje);
      return false;
    }
  }

  async function responder(invitacion: InvitacionAmistadDTO, aceptar: boolean) {
    setError("");
    try {
      await responderInvitacionAmistad(invitacion.id, aceptar);
      await cargarAmistades();
    } catch (cause: unknown) {
      const mensaje = mensajeDeError(
        cause,
        "No se pudo responder la invitación.",
      );
      setError(mensaje);
    }
  }

  async function eliminar(nombreUsuario: string) {
    setError("");
    try {
      await eliminarAmistad(nombreUsuario);
      await cargarAmistades();
    } catch (cause: unknown) {
      const mensaje = mensajeDeError(
        cause,
        "No se pudo eliminar la amistad.",
      );
      setError(mensaje);
    }
  }

  return {
    amistades,
    usuarioEncontrado,
    cargando,
    error,
    buscar,
    invitar,
    responder,
    eliminar,
    recargar: cargarAmistades,
  };
}
