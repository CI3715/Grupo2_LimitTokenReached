"use client";

import { useAuthSession } from "@/features/auth/useAuthSession";

export function AuthSessionIndicator() {
  const { usuario, mensajeSesion, salir } = useAuthSession();

  if (!usuario && !mensajeSesion) return null;

  return (
    <aside className="auth-session-indicator" aria-live="polite">
      {usuario ? (
        <>
          <span>Sesión: {usuario.nombre_visible}</span>
          <button type="button" onClick={salir}>
            Cerrar sesión
          </button>
        </>
      ) : (
        <span>{mensajeSesion}</span>
      )}
    </aside>
  );
}
