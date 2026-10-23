"use client";

import { useCallback, useEffect, useState } from "react";
import { listCategories } from "@/services/listCategories";
import type { CategoriaDTO } from "@/types/movements";

export function useMovementCategories() {
  const [categories, setCategories] = useState<CategoriaDTO[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setCategories(await listCategories());
    } catch (caught: unknown) {
      console.error("No se pudieron cargar las categorías.", caught);
      setError("No se pudieron cargar las categorías.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    void listCategories()
      .then((nextCategories) => {
        if (active) setCategories(nextCategories);
      })
      .catch((caught: unknown) => {
        console.error("No se pudieron cargar las categorías.", caught);
        if (active) setError("No se pudieron cargar las categorías.");
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  return { categories, loading, error, refresh };
}
