"use client";

import { useState } from "react";
import Link from "next/link";
import { HashDisplay } from "@/components/HashDisplay";
import { SarIcon } from "@/components/SarIcon";
import { Ltr } from "@/components/Ltr";
import { useLocale } from "@/lib/i18n/LocaleProvider";
import { statusForScore, statusClassName, statusLabel } from "@/lib/status";

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3000";

type DecisionResponse = {
  receipt_id: string;
  decision_id: string;
  output: { prob_good: number; score: number };
};

export default function DemoPage() {
  const { dict: t } = useLocale();
  const [income, setIncome] = useState(90000);
  const [debtRatio, setDebtRatio] = useState(0.23);
  const [missedPayments, setMissedPayments] = useState(0);
  const [creditHistoryMonths, setCreditHistoryMonths] = useState(48);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<DecisionResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const payload = {
    income,
    debt_ratio: Number(debtRatio.toFixed(2)),
    missed_payments: missedPayments,
    credit_history_months: creditHistoryMonths,
  };

  const curlCommand = `curl -X POST ${API_BASE}/decision \\
  -H "Content-Type: application/json" \\
  -d '${JSON.stringify(payload)}'`;

  async function submit() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/decision`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
      setResult(await res.json());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Request failed");
      setResult(null);
    } finally {
      setLoading(false);
    }
  }

  const status = result ? statusForScore(result.output.score) : null;
  const numericInputClass =
    "w-full box-border bg-surface-alt border border-border rounded-md py-2.5 px-3 font-mono text-[13.5px] text-ink outline-none focus:border-hairline";

  return (
    <div>
      <div className="mb-7">
        <h1 className="m-0 mb-1.5 text-[22px] font-semibold text-ink">{t.demo.title}</h1>
        <p className="m-0 text-[13.5px] text-faint">{t.demo.subtitle}</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
        <div className="rounded-lg border border-border bg-surface p-6">
          <div className="text-[13px] font-semibold text-ink mb-[18px]">{t.demo.application}</div>

          <div className="mb-4">
            <label className="flex items-center gap-1.5 text-[11.5px] text-faint mb-1.5">
              {t.demo.incomeLabel}
              <SarIcon size={11} className="text-faint" />
            </label>
            <div className="relative">
              <SarIcon
                size={13}
                className="absolute top-1/2 -translate-y-1/2 start-3 text-ghost pointer-events-none"
              />
              <input
                dir="ltr"
                type="number"
                value={income}
                onChange={(e) => setIncome(Number(e.target.value) || 0)}
                className={`${numericInputClass} ps-8`}
              />
            </div>
          </div>

          <div className="mb-4">
            <label className="block text-[11.5px] text-faint mb-1.5">
              {t.demo.debtLabel(Math.round(debtRatio * 100))}
            </label>
            <input
              dir="ltr"
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={debtRatio}
              onChange={(e) => setDebtRatio(Number(e.target.value))}
              className="w-full accent-accent"
            />
          </div>

          <div className="mb-4">
            <label className="block text-[11.5px] text-faint mb-1.5">{t.demo.missedLabel}</label>
            <input
              dir="ltr"
              type="number"
              min={0}
              max={12}
              value={missedPayments}
              onChange={(e) => setMissedPayments(Number(e.target.value) || 0)}
              className={numericInputClass}
            />
          </div>

          <div className="mb-[22px]">
            <label className="block text-[11.5px] text-faint mb-1.5">{t.demo.historyLabel}</label>
            <input
              dir="ltr"
              type="number"
              min={0}
              max={480}
              value={creditHistoryMonths}
              onChange={(e) => setCreditHistoryMonths(Number(e.target.value) || 0)}
              className={numericInputClass}
            />
          </div>

          <button
            onClick={submit}
            disabled={loading}
            className="w-full py-3 rounded-md border border-accent bg-accent/10 text-accent text-[13.5px] font-semibold cursor-pointer hover:bg-accent/20 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            {loading ? t.demo.submitting : t.demo.submit}
          </button>

          {error && (
            <div className="mt-4 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-[12.5px] text-danger">
              {error}
            </div>
          )}

          <div className="mt-[22px] mb-2 text-[11px] tracking-[0.04em] uppercase text-ghost">
            {t.demo.equivalentRequest}
          </div>
          <Ltr className="block text-start bg-surface-alt border border-border rounded-md px-4 py-3.5 font-mono text-[11.5px] text-mute leading-relaxed whitespace-pre-wrap break-all">
            {curlCommand}
          </Ltr>
        </div>

        <div className="rounded-lg border border-border bg-surface p-6 min-h-[400px]">
          <div className="text-[13px] font-semibold text-ink mb-[18px]">{t.demo.result}</div>

          {!result ? (
            <div className="flex items-center justify-center h-80 text-hairline text-[13px] text-center">
              {t.demo.emptyResult}
            </div>
          ) : (
            <>
              <div className="flex gap-6 mb-5">
                <div>
                  <div className="text-[11px] tracking-[0.04em] uppercase text-ghost mb-1.5">{t.demo.score}</div>
                  <Ltr className="block font-mono text-[32px] font-semibold text-ink">{result.output.score}</Ltr>
                </div>
                <div>
                  <div className="text-[11px] tracking-[0.04em] uppercase text-ghost mb-1.5">{t.demo.pGood}</div>
                  <Ltr className="block font-mono text-[32px] font-semibold text-ink">
                    {Math.round(result.output.prob_good * 100)}%
                  </Ltr>
                </div>
                <div className="flex-1 flex items-start justify-end">
                  <span
                    className={`inline-block px-[11px] py-[5px] rounded text-[11px] font-medium tracking-[0.03em] uppercase border ${statusClassName(status!)}`}
                  >
                    {statusLabel(status!, t)}
                  </span>
                </div>
              </div>

              <div className="h-px bg-border mb-[18px]" />

              <div className="text-[11px] tracking-[0.04em] uppercase text-ghost mb-2.5">{t.demo.signedReceipt}</div>
              <div className="flex flex-col gap-2.5 mb-[18px]">
                <div className="flex justify-between text-[12.5px]">
                  <span className="text-faint">{t.demo.receiptIdLabel}</span>
                  <HashDisplay value={result.receipt_id} chars={8} />
                </div>
                <div className="flex justify-between text-[12.5px]">
                  <span className="text-faint">{t.demo.decisionIdLabel}</span>
                  <HashDisplay value={result.decision_id} chars={8} />
                </div>
              </div>

              <Link
                href={`/audit/${result.receipt_id}`}
                className="inline-flex items-center gap-1.5 text-[13px] text-accent hover:text-accent-soft font-medium transition-colors"
              >
                {t.demo.viewFullReceipt}
              </Link>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
