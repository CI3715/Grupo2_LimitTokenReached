type InfoCardProps = {
  label: string;
  value: string;
};

export function InfoCard({ label, value }: InfoCardProps) {
  return (
    <div className="data-item min-w-0 rounded-2xl p-4">
      <p className="data-label text-[10px] font-medium uppercase tracking-[0.2em]">
        {label}
      </p>
      <p className="data-value mt-3 truncate font-mono text-sm font-medium" title={value}>
        {value}
      </p>
    </div>
  );
}
