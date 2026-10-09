import type {ConnectionStatus} from "./tipes";
import { CSSProperties } from "react";

function StatusOrb({
  status,
  latency,
}: {
  status: ConnectionStatus;
  latency?: number;
}) {
  const duration = latency
    ? Math.max(
        1.4,
        Math.min(
          3.4,
          latency / 55,
        ),
      )
    : 2.4;

  const style = {
    "--orb-duration":
      `${duration}s`,
  } as CSSProperties;

  return (
    <div
      className={`orb orb-${status}`}
      style={style}
    >
      <div className="orb-aura" />

      <div className="orb-ring orb-ring-one" />

      <div className="orb-ring orb-ring-two" />

      <div className="orb-ring orb-ring-three" />

      <div className="orb-particle orb-particle-one" />

      <div className="orb-particle orb-particle-two" />

      <div className="orb-particle orb-particle-three" />

      <div className="orb-shell">
        <div className="orb-shell-highlight" />

        <div className="orb-core">
          <div className="orb-core-inner" />
        </div>
      </div>

      {status ===
        "loading" && (
        <>
          <div className="orb-wave orb-wave-one" />

          <div className="orb-wave orb-wave-two" />
        </>
      )}
    </div>
  );
}

export default StatusOrb;