type Props = {
  label: string;
  value: React.ReactNode;
  sub?: React.ReactNode;
};

export function StatCard({ label, value, sub }: Props) {
  return (
    <div className="rounded-lg border border-border bg-surface px-5 py-[18px]">
      <div className="text-[11px] tracking-[0.04em] uppercase text-ghost mb-2.5">
        {label}
      </div>
      <div className="font-mono text-[26px] font-semibold text-ink truncate">
        {value}
      </div>
      {sub && <div className="text-[12px] text-faint mt-1">{sub}</div>}
    </div>
  );
}
