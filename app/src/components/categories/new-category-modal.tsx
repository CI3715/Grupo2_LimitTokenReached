"use client";

import { useEffect, useRef, useState, type FormEvent } from "react";
import type {
  CategoriaDTO,
  CrearCategoriaRequest,
  TipoCategoria,
} from "@/types/categories";

type Props = {
  categories: CategoriaDTO[];
  onClose: () => void;
  onCreate: (request: CrearCategoriaRequest) => Promise<CategoriaDTO>;
};

function errorMessage(error: unknown): string {
  let value = error;
  if (typeof value === "string") {
    const text = value;
    try {
      value = JSON.parse(text);
    } catch {
      return text.trim() || "No se pudo crear la categoría.";
    }
  }

  if (typeof value === "object" && value !== null && "mensaje" in value) {
    const message = value.mensaje;
    if (typeof message === "string") return message;
  }

  return "No se pudo crear la categoría. Intenta nuevamente.";
}

export function NewCategoryModal({ categories, onClose, onCreate }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [tipo, setTipo] = useState<TipoCategoria>("Gasto");
  const [nombre, setNombre] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const node = dialog.current;
    const previousFocus = document.activeElement as HTMLElement | null;
    node?.showModal();
    node?.querySelector<HTMLInputElement>('[name="nombre"]')?.focus();
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      node?.close();
      document.body.style.overflow = previousOverflow;
      previousFocus?.focus();
    };
  }, []);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const cleanName = nombre.trim();
    if (!cleanName) {
      setError("El nombre de la categoría no puede estar vacío.");
      return;
    }

    const normalizedName = cleanName.toLowerCase();
    const duplicate = categories.some(
      (category) =>
        category.tipo === tipo &&
        category.nombre.trim().toLowerCase() === normalizedName,
    );
    if (duplicate) {
      setError("Ya existe una categoría con ese nombre para ese tipo.");
      return;
    }

    setPending(true);
    setError(null);
    try {
      await onCreate({ nombre: cleanName, tipo });
      onClose();
    } catch (caught: unknown) {
      setError(errorMessage(caught));
    } finally {
      setPending(false);
    }
  }

  return (
    <dialog
      ref={dialog}
      className="new-category-dialog"
      aria-labelledby="new-category-title"
      onCancel={(event) => {
        event.preventDefault();
        if (!pending) onClose();
      }}
      onClick={(event) => {
        if (event.target !== event.currentTarget || pending) return;
        const bounds = event.currentTarget.getBoundingClientRect();
        if (
          event.clientX < bounds.left ||
          event.clientX > bounds.right ||
          event.clientY < bounds.top ||
          event.clientY > bounds.bottom
        ) {
          onClose();
        }
      }}
    >
      <header>
        <div>
          <p className="eyebrow">Nueva categoría</p>
          <h2 id="new-category-title">Crea una categoría</h2>
          <p>Organiza tus movimientos con una categoría propia.</p>
        </div>
        <button
          type="button"
          className="close-account-dialog"
          aria-label="Cerrar formulario"
          onClick={onClose}
          disabled={pending}
        >
          
          ×

        </button>
      </header>
      <form onSubmit={(event) => void submit(event)} aria-busy={pending}>
        {error && (
          <p id="new-category-error" className="account-error" role="alert">
            {error}
          </p>
        )}
        <label htmlFor="new-category-name">Nombre</label>
        <input
          id="new-category-name"
          name="nombre"
          aria-invalid={error ? true : undefined}
          aria-describedby={error ? "new-category-error" : undefined}
          value={nombre}
          onChange={(event) => {
            setNombre(event.target.value);
            setError(null);
          }}
          placeholder="Ej. Mascotas"
          autoComplete="off"
          maxLength={80}
          required
          disabled={pending}
        />
        <fieldset className="category-type-options" disabled={pending}>
          <legend>Tipo</legend>
          {(["Gasto", "Ingreso"] as const).map((value) => (
            <label key={value} className={tipo === value ? "selected" : ""}>
              <input
                type="radio"
                name="tipo"
                value={value}
                checked={tipo === value}
                onChange={() => {
                  setTipo(value);
                  setError(null);
                }}
              />
              {value}
            </label>
          ))}
        </fieldset>
        <footer>
          <button
            type="button"
            className="account-secondary"
            onClick={onClose}
            disabled={pending}
          >
            Cancelar
          </button>
          <button className="account-primary" type="submit" disabled={pending}>
            {pending ? "Creando..." : "Crear categoría"}
          </button>
        </footer>
      </form>
    </dialog>
  );
}
