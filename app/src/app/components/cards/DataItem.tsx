function DataItem({
  label,
  value,
  accent = false,
}: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div className="data-item group relative overflow-hidden rounded-2xl p-4 transition duration-300">
      <div className="data-item-highlight absolute inset-x-0 top-0 h-px opacity-0 transition group-hover:opacity-100" />

      <p className="data-label text-[10px] font-medium uppercase tracking-[0.2em]">
        {label}
      </p>

      <p
        className={`mt-3 truncate font-mono text-sm font-medium ${
          accent
            ? "data-value-accent"
            : "data-value"
        }`}
      >
        {value}
      </p>
    </div>
  );
}

export default DataItem;