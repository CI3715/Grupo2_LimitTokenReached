function FlowNode({
  title,
  subtitle,
  active,
}: {
  title: string;
  subtitle: string;
  active: boolean;
}) {
  return (
    <div className="relative z-10 flex min-w-0 flex-1 flex-col items-center">
      <div
        className={`flow-node ${
          active
            ? "flow-node-active"
            : ""
        }`}
      >
        <div className="flow-node-inner" />
      </div>

      <span className="flow-title mt-3 text-[11px] font-semibold tracking-[0.04em] sm:text-xs">
        {title}
      </span>

      <span className="flow-subtitle mt-1 hidden text-[9px] uppercase tracking-[0.18em] sm:block">
        {subtitle}
      </span>
    </div>
  );
}

export default FlowNode;