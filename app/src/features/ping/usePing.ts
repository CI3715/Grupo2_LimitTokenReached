"use client";

import { useCallback, useState } from "react";

import { pingServer } from "@/services/pingService";
import type { ConnectionState } from "@/types/ping";

export function usePing() {
  const [state, setState] = useState<ConnectionState>({
    status: "idle",
  });

  const checkConnection = useCallback(async () => {
    if (state.status === "loading") {
      return;
    }

    setState({ status: "loading" });

    try {
      const data = await pingServer();
      setState({ status: "success", data });
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : "No se pudo verificar la conexión con el servidor.";

      setState({ status: "error", message });
    }
  }, [state.status]);

  return { state, checkConnection };
}
