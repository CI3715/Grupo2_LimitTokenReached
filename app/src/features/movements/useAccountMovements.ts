"use client";

import { useCallback, useEffect, useState } from "react";
import { listMovements } from "@/services/listMovements";
import type { MovimientoDTO } from "@/types/movements";

export function useAccountMovements(accountId: string | null) {
  const [movements, setMovements] = useState<MovimientoDTO[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!accountId) {
      setMovements([]);
      return [];
    }

    try {
      setError(null);
      const nextMovements = await listMovements(accountId);
      setMovements(nextMovements);
      return nextMovements;
    } catch (error: unknown) {
      console.error("No se pudieron cargar los movimientos de la cuenta.", error);
      setError("No se pudieron cargar los movimientos de la cuenta.");
      throw error;
    }
  }, [accountId]);

  useEffect(() => {
    async function loadMovements() {
      if (!accountId) {
        setMovements([]);
        return;
      }

      try {
        setError(null);
        setMovements(await listMovements(accountId));
      } catch (error: unknown) {
        console.error("No se pudieron cargar los movimientos de la cuenta.", error);
        setError("No se pudieron cargar los movimientos de la cuenta.");
      }
    }

    void loadMovements();
  }, [accountId]);

  return {
    movements,
    error,
    refresh,
  };
}
