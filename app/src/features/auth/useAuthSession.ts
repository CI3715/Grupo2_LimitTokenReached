"use client";

import { isTauri } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

import {
  cerrarSesion,
  obtenerPerfil,
} from "@/services/autenticacion";
import type { UsuarioDTO } from "@/types/autenticacion";

export function useAuthSession() {
  const [usuario, setUsuario] = useState<UsuarioDTO | null>(null);
  const [mensajeSesion, setMensajeSesion] = useState("");

  const cargarPerfil = useCallback(() => {
    if (!isTauri()) return;

    void obtenerPerfil()
      .then((perfil) => {
        setUsuario(perfil);
        setMensajeSesion("");
      })
      .catch((error: unknown) => {
        const mensaje =
          typeof error === "string"
            ? error
            : "No se pudo consultar la sesión.";
        setUsuario(null);
        setMensajeSesion(
          mensaje.includes("Tu sesión venció")
            ? "Tu sesión venció, inicia sesión de nuevo."
            : "",
        );
      });
  }, []);

  useEffect(() => {
    cargarPerfil();
    window.addEventListener("auth-session-changed", cargarPerfil);
    return () => window.removeEventListener("auth-session-changed", cargarPerfil);
  }, [cargarPerfil]);

  async function salir() {
    try {
      await cerrarSesion();
      setUsuario(null);
      setMensajeSesion("");
    } catch {
      setMensajeSesion("No se pudo cerrar la sesión en este dispositivo.");
    }
  }

  return { usuario, mensajeSesion, salir };
}
