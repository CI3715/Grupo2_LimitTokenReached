type LatencyBadgeProps = {
  latency?: number;
};

export function LatencyBadge({ latency }: LatencyBadgeProps) {
  return (
    <span className="data-value-accent rounded-lg px-3 py-1 font-mono text-sm font-semibold">
      {latency === undefined ? "--" : `${latency} ms`}
    </span>
  );
}
