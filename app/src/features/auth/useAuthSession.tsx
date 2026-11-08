"use client";

import { isTauri } from "@tauri-apps/api/core";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";

import {
  cerrarSesion,
  obtenerPerfil,
} from "@/services/autenticacion";
import type { UsuarioDTO } from "@/types/autenticacion";

type AuthSessionValue = {
  usuario: UsuarioDTO | null;
  cargando: boolean;
  errorPerfil: string;
  esAplicacionTauri: boolean;
  mensajeSesion: string;
  cargarPerfil: () => void;
  salir: () => Promise<void>;
};

const AuthSessionContext = createContext<AuthSessionValue | null>(null);

export function AuthSessionProvider({ children }: { children: ReactNode }) {
  const [usuario, setUsuario] = useState<UsuarioDTO | null>(null);
  const [cargando, setCargando] = useState(true);
  const [errorPerfil, setErrorPerfil] = useState("");
  const [esAplicacionTauri, setEsAplicacionTauri] = useState(false);
  const [mensajeSesion, setMensajeSesion] = useState("");

  const cargarPerfil = useCallback(() => {
    setErrorPerfil("");
    setMensajeSesion("");
    const disponible = isTauri();
    setEsAplicacionTauri(disponible);
    if (!disponible) {
      setUsuario(null);
      setCargando(false);
      setErrorPerfil(
        "El perfil solo se puede consultar desde la aplicación de escritorio.",
      );
      return;
    }

    setCargando(true);
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
        setErrorPerfil(mensaje);
        setMensajeSesion(
          mensaje.includes("Tu sesión venció")
            ? "Tu sesión venció, inicia sesión de nuevo."
            : "",
        );
      })
      .finally(() => setCargando(false));
  }, []);

  useEffect(() => {
    void Promise.resolve().then(cargarPerfil);
    window.addEventListener("auth-session-changed", cargarPerfil);
    return () => window.removeEventListener("auth-session-changed", cargarPerfil);
  }, [cargarPerfil]);

  async function salir() {
    try {
      await cerrarSesion();
      setUsuario(null);
      setErrorPerfil("No hay una sesión iniciada.");
      setMensajeSesion("");
    } catch {
      const mensaje = "No se pudo cerrar la sesión en este dispositivo.";
      setErrorPerfil(mensaje);
      setMensajeSesion(mensaje);
    }
  }

  return (
    <AuthSessionContext.Provider
      value={{
        usuario,
        cargando,
        errorPerfil,
        esAplicacionTauri,
        mensajeSesion,
        cargarPerfil,
        salir,
      }}
    >
      {children}
    </AuthSessionContext.Provider>
  );
}

export function useAuthSession() {
  const contexto = useContext(AuthSessionContext);
  if (!contexto) {
    throw new Error("useAuthSession debe usarse dentro de AuthSessionProvider.");
  }
  return contexto;
}
