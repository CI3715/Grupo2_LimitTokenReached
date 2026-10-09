import type {ConnectionStatus} from "./tipes";
import FlowNode from "./FlowNode";

function SystemFlow({
  status,
}: {
  status: ConnectionStatus;
}) {
  const firstActive =
    status === "loading" ||
    status === "connected";

  const secondActive =
    status === "connected";

  return (
    <div className="relative mx-auto mt-10 w-full max-w-xl">
      <div className="flow-base-line absolute left-[16.6%] right-[16.6%] top-[14px] h-px" />

      <div
        className={`signal-line ${
          firstActive
            ? "signal-line-active"
            : ""
        }`}
      />

      <div
        className={`signal-tracer ${
          status ===
          "loading"
            ? "signal-tracer-running"
            : status ===
                "connected"
              ? "signal-tracer-complete"
              : ""
        }`}
      />

      <div className="relative flex items-start justify-between">
        <FlowNode
          title="Next.js"
          subtitle="Interface"
          active={
            firstActive
          }
        />

        <FlowNode
          title="Tauri / Rust"
          subtitle="Bridge"
          active={
            firstActive
          }
        />

        <FlowNode
          title="FastAPI"
          subtitle="Server"
          active={
            secondActive
          }
        />
      </div>
    </div>
  );
}

export default SystemFlow;