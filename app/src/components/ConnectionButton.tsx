"use client";

type ConnectionButtonProps = {
  isLoading: boolean;
  isConnected: boolean;
  onCheckConnection: () => void;
};

export function ConnectionButton({
  isLoading,
  isConnected,
  onCheckConnection,
}: ConnectionButtonProps) {
  return (
    <button
      type="button"
      disabled={isLoading}
      onClick={onCheckConnection}
      className="tech-button group relative flex w-full max-w-sm items-center justify-between overflow-hidden rounded-2xl px-5 py-4 text-left disabled:cursor-not-allowed"
    >
      <div className="relative z-10">
        <p className="button-eyebrow text-[10px] uppercase tracking-[0.22em]">
          Acción del sistema
        </p>

        <p className="primary-text mt-1 text-sm font-semibold">
          {isLoading
            ? "Estableciendo conexión..."
            : isConnected
              ? "Verificar nuevamente"
              : "Verificar conexión"}
        </p>
      </div>

      <div className="button-arrow relative z-10 flex h-10 w-10 items-center justify-center rounded-xl">
        {isLoading ? (
          <div className="loading-spinner h-4 w-4 animate-spin rounded-full" />
        ) : (
          <span className="primary-text transition duration-300 group-hover:translate-x-1">
            →
          </span>
        )}
      </div>
    </button>
  );
}
