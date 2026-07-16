"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { listReceipts, type Receipt } from "@/lib/api";
import { HashDisplay } from "@/components/HashDisplay";
import { Ltr } from "@/components/Ltr";
import { useLocale } from "@/lib/i18n/LocaleProvider";
import { formatRiyadhStamp } from "@/lib/i18n/format";
import { statusForScore, statusClassName, statusLabel } from "@/lib/status";

const PAGE_SIZE = 8;
const FETCH_LIMIT = 100;
const COLS = "grid-cols-[1fr_1fr_0.95fr_0.7fr_0.55fr_0.95fr_0.4fr]";

export default function AuditCenter() {
  const { locale, dict: t } = useLocale();
  const [receipts, setReceipts] = useState<Receipt[] | null>(null);
  const [total, setTotal] = useState(0);
  const [error, setError] = useState(false);
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);

  useEffect(() => {
    listReceipts(FETCH_LIMIT, 0)
      .then((data) => {
        setReceipts(data.receipts);
        setTotal(data.total);
      })
      .catch(() => setError(true));
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!receipts) return [];
    if (!q) return receipts;
    return receipts.filter(
      (r) =>
        r.receipt_id.toLowerCase().includes(q) ||
        r.decision_id.toLowerCase().includes(q) ||
        r.model_id.toLowerCase().includes(q) ||
        r.policy_id.toLowerCase().includes(q) ||
        r.input_hash.toLowerCase().includes(q) ||
        r.output_hash.toLowerCase().includes(q) ||
        r.model_hash.toLowerCase().includes(q) ||
        r.policy_hash.toLowerCase().includes(q)
    );
  }, [receipts, query]);

  const totalPages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const currentPage = Math.min(page, totalPages - 1);
  const pageRows = filtered.slice(currentPage * PAGE_SIZE, currentPage * PAGE_SIZE + PAGE_SIZE);

  return (
    <div>
      <div className="mb-7">
        <h1 className="m-0 mb-1.5 text-[22px] font-semibold text-ink">{t.audit.title}</h1>
        <p className="m-0 text-[13.5px] text-faint">
          {t.audit.subtitle}
          {total > FETCH_LIMIT && t.audit.showingMostRecent(FETCH_LIMIT, total)}
        </p>
      </div>

      <div className="flex items-center gap-2.5 mb-4">
        <input
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setPage(0);
          }}
          placeholder={t.audit.searchPlaceholder}
          className="flex-1 max-w-[340px] bg-surface border border-border rounded-md px-3 py-2.5 font-mono text-[12.5px] text-ink outline-none focus:border-hairline"
        />
        <div className="flex-1" />
        <div className="text-[12px] text-ghost font-mono">{t.audit.receiptsCount(filtered.length)}</div>
      </div>

      <div className="rounded-lg border border-border bg-surface overflow-hidden">
        {error ? (
          <div className="px-5 py-10 text-center text-ghost text-[13px]">{t.audit.backendUnreachable}</div>
        ) : !receipts ? (
          <div className="px-5 py-10 text-center text-ghost text-[13px]">{t.audit.loading}</div>
        ) : (
          <>
            <div
              className={`grid ${COLS} px-5 py-2.5 text-[11px] text-ghost border-b border-border ${
                locale === "en" ? "tracking-[0.04em] uppercase" : "tracking-normal"
              }`}
            >
              <div>{t.audit.colReceiptId}</div>
              <div>{t.audit.colDecisionId}</div>
              <div>{t.common.colModel}</div>
              <div>{t.common.colStatus}</div>
              <div>{t.common.colScore}</div>
              <div>{t.audit.colTimestamp}</div>
              <div />
            </div>
            {pageRows.map((r) => {
              const status = typeof r.output?.score === "number" ? statusForScore(r.output.score) : null;
              return (
                <div
                  key={r.receipt_id}
                  className={`grid ${COLS} items-center px-5 py-3 text-[13px] border-b border-border-soft last:border-0 hover:bg-surface-hover`}
                >
                  <HashDisplay value={r.receipt_id} chars={6} />
                  <HashDisplay value={r.decision_id} chars={6} />
                  <div className="min-w-0">
                    {r.model_name ? (
                      <span className="font-mono text-[12px] text-dim truncate block" title={r.model_name}>
                        {r.model_name}
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
                  <Link
                    href={`/audit/${r.receipt_id}`}
                    className="text-[12.5px] text-mute hover:text-ink transition-colors"
                  >
                    {t.audit.view}
                  </Link>
                </div>
              );
            })}
            {pageRows.length === 0 && (
              <div className="px-5 py-10 text-center text-ghost text-[13px]">{t.audit.noMatch}</div>
            )}
          </>
        )}
      </div>

      {receipts && filtered.length > 0 && (
        <div className="flex items-center justify-between mt-4">
          <div className="text-[12px] text-ghost font-mono">
            {t.audit.page(currentPage + 1, totalPages)}
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              disabled={currentPage === 0}
              className="px-3.5 py-[7px] rounded-md text-[12.5px] border border-border bg-surface text-dim disabled:text-hairline disabled:cursor-default cursor-pointer"
            >
              {t.audit.prev}
            </button>
            <button
              onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
              disabled={currentPage >= totalPages - 1}
              className="px-3.5 py-[7px] rounded-md text-[12.5px] border border-border bg-surface text-dim disabled:text-hairline disabled:cursor-default cursor-pointer"
            >
              {t.audit.next}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
