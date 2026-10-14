import type { ConnectionState } from "@/types/ping";

type StatusIndicatorProps = {
  status: ConnectionState["status"];
  compact?: boolean;
};

const statusCopy: Record<ConnectionState["status"], string> = {
  idle: "Standby",
  loading: "Connecting",
  success: "Online",
  error: "Error",
};

export function StatusIndicator({ status, compact = false }: StatusIndicatorProps) {
  return (
    <div
      className={`status-pill flex items-center gap-2 rounded-full px-3 py-2 text-[10px] uppercase tracking-[0.14em] ${
        compact ? "sm:px-2" : ""
      }`}
      role="status"
      aria-label={`Estado: ${statusCopy[status]}`}
    >
      <span
        className={`h-1.5 w-1.5 rounded-full ${
          status === "success"
            ? "bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.8)]"
            : status === "loading"
              ? "animate-pulse bg-cyan-400"
              : status === "error"
                ? "bg-red-400"
                : "standby-dot"
        }`}
      />
      {statusCopy[status]}
    </div>
  );
}
