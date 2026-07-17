import Link from "next/link";
import { getStats, listReceipts } from "@/lib/api";
import { StatCard } from "@/components/StatCard";
import { HashDisplay } from "@/components/HashDisplay";
import { Ltr } from "@/components/Ltr";
import { getDict } from "@/lib/i18n/server";
import { formatRiyadhStamp, formatCount } from "@/lib/i18n/format";
import { statusForScore, statusClassName, statusLabel } from "@/lib/status";

const COLS = "grid-cols-[1.05fr_1.05fr_0.95fr_0.7fr_0.55fr_0.9fr_0.4fr]";

export default async function Dashboard() {
  const { dict: t } = await getDict();
  const [stats, recent] = await Promise.all([
    getStats().catch(() => null),
    listReceipts(5, 0).catch(() => null),
  ]);

  return (
    <div>
      <div className="mb-8">
        <h1 className="m-0 mb-1.5 text-[22px] font-semibold text-ink">{t.dashboard.title}</h1>
        <p className="m-0 text-[13.5px] text-faint">{t.dashboard.subtitle}</p>
      </div>

      {stats ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5 mb-10">
          <StatCard label={t.dashboard.totalDecisions} value={<Ltr>{formatCount(stats.total_receipts)}</Ltr>} />
          <StatCard
            label={t.dashboard.modelId}
            value={stats.model_name ? <Ltr>{stats.model_name}</Ltr> : <HashDisplay value={stats.model_id} chars={8} />}
            sub={stats.model_name && (stats.model_version ?? <HashDisplay value={stats.model_id} chars={6} />)}
          />
          <StatCard label={t.dashboard.modelHash} value={<HashDisplay value={stats.model_hash} chars={6} />} />
          <StatCard label={t.dashboard.policyHash} value={<HashDisplay value={stats.policy_hash} chars={6} />} />
        </div>
      ) : (
        <div className="rounded-lg border border-border bg-surface px-6 py-8 mb-10 text-center text-ghost text-sm">
          {t.dashboard.backendUnreachable}
        </div>
      )}

      <div className="flex items-baseline justify-between mb-3.5">
        <h2 className="m-0 text-[15px] font-semibold text-ink">{t.dashboard.recentDecisions}</h2>
        <Link href="/audit" className="text-[12.5px] text-mute hover:text-ink transition-colors">
          {t.dashboard.viewAll}
        </Link>
      </div>

      {recent && recent.receipts.length > 0 ? (
        <div className="rounded-lg border border-border bg-surface overflow-hidden">
          <div className={`grid ${COLS} px-5 py-2.5 text-[11px] tracking-[0.04em] uppercase text-ghost border-b border-border`}>
            <div>{t.dashboard.colReceiptId}</div>
            <div>{t.dashboard.colDecisionId}</div>
            <div>{t.common.colModel}</div>
            <div>{t.common.colStatus}</div>
            <div>{t.common.colScore}</div>
            <div>{t.dashboard.colTimestamp}</div>
            <div />
          </div>
          {recent.receipts.map((r) => {
            const status = typeof r.output?.score === "number" ? statusForScore(r.output.score) : null;
            return (
              <Link
                key={r.receipt_id}
                href={`/audit/${r.receipt_id}`}
                className={`group grid ${COLS} items-center px-5 py-3 text-[13px] border-b border-border-soft last:border-0 hover:bg-surface-hover cursor-pointer`}
              >
                <HashDisplay value={r.receipt_id} chars={6} />
                <HashDisplay value={r.decision_id} chars={6} />
                <div className="min-w-0">
                  {r.model_name ? (
                    <span
                      className="font-mono text-[12px] text-dim truncate block"
                      title={r.model_version ? `${r.model_name} ${r.model_version}` : r.model_name}
                    >
                      {r.model_name}
                      {r.model_version && <span className="text-ghost"> {r.model_version}</span>}
                    </span>
                  ) : (
                    <HashDisplay value={r.model_id} chars={6} />
                  )}
                </div>
                <div>
                  {status ? (
                    <span
                      className={`inline-block px-[9px] py-[3px] rounded text-[11px] font-medium tracking-[0.03em] uppercase border ${statusClassName(status)}`}
                    >
                      {statusLabel(status, t)}
                    </span>
                  ) : (
                    <span className="text-ghost">—</span>
                  )}
                </div>
                <div className="font-mono text-dim">
                  <Ltr>{r.output?.score ?? "—"}</Ltr>
                </div>
                <div className="font-mono text-[12px] text-faint">
                  <Ltr>{formatRiyadhStamp(r.timestamp)}</Ltr> {t.common.riyadhTimeShort}
                </div>
                <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md border border-border text-[11.5px] text-mute group-hover:text-ink group-hover:border-hairline transition-colors justify-self-end">
                  {t.dashboard.view}
                </span>
              </Link>
            );
          })}
        </div>
      ) : (
        <div className="rounded-lg border border-border bg-surface px-6 py-8 text-center text-ghost text-sm">
          {t.dashboard.noDecisions}
        </div>
      )}
    </div>
  );
}
