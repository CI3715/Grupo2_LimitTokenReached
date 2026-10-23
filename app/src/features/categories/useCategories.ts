"use client";

import { useCallback, useEffect, useState } from "react";
import { createCategory } from "@/services/createCategory";
import { listCategories } from "@/services/listCategories";
import type { CategoriaDTO, CrearCategoriaRequest } from "@/types/categories";

function mergeCategories(
  current: CategoriaDTO[],
  incoming: CategoriaDTO[],
): CategoriaDTO[] {
  const byId = new Map(current.map((category) => [category.id, category]));
  for (const category of incoming) byId.set(category.id, category);
  return [...byId.values()];
}

export function useCategories() {
  const [categories, setCategories] = useState<CategoriaDTO[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const nextCategories = await listCategories();
      setCategories((current) => mergeCategories(current, nextCategories));
    } catch (caught: unknown) {
      console.error("No se pudieron cargar las categorías.", caught);
      setError("No se pudieron cargar las categorías.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    async function load() {
      try {
        const initialCategories = await listCategories();
        if (active) {
          setCategories((current) => mergeCategories(current, initialCategories));
        }
      } catch (caught: unknown) {
        console.error("No se pudieron cargar las categorías.", caught);
        if (active) setError("No se pudieron cargar las categorías.");
      } finally {
        if (active) setLoading(false);
      }
    }

    void load();
    return () => {
      active = false;
    };
  }, []);

  async function create(request: CrearCategoriaRequest) {
    const category = await createCategory(request);
    setCategories((current) =>
      current.some((item) => item.id === category.id)
        ? current
        : [...current, category],
    );
    return category;
  }

  return { categories, loading, error, refresh, create };
}
