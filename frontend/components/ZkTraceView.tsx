"use client";

import { Ltr } from "@/components/Ltr";
import { useT } from "@/lib/i18n/LocaleProvider";
import { formatRiyadhStamp } from "@/lib/i18n/format";
import type { ZkTrace } from "@/lib/api";

// Feature order fixed by backend/src/execution/inference/mod.rs — must stay
// in sync. circuit_public_values echoes these back (indices 0-3), then the
// circuit's own predicted class (4) and class probabilities (5, 6).
const FEATURE_LABELS = ["income", "debt_ratio", "missed_payments", "credit_history_months"] as const;

const FEATURE_LABEL_KEYS = {
  income: "incomeLabel",
  debt_ratio: "debtRatioFieldLabel",
  missed_payments: "missedLabel",
  credit_history_months: "historyLabel",
} as const;

function SourceBadge({ trace }: { trace: ZkTrace }) {
  const t = useT();
  const phaseLabel = trace.phase === "prove" ? t.trace.phaseProve : t.trace.phaseVerify;
  const sourceLabel =
    trace.source === "gateway" ? t.trace.sourceGateway : trace.source === "lookup" ? t.trace.sourceLookup : t.trace.sourcePasted;

  return (
    <div className="flex items-center gap-2 flex-wrap">
      <span
        className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-[11px] font-medium border ${
          trace.success ? "text-verified bg-verified/10 border-verified/35" : "text-danger bg-danger/10 border-danger/35"
        }`}
      >
        <span className={`w-[6px] h-[6px] rounded-full ${trace.success ? "bg-verified" : "bg-danger"}`} />
        {trace.success ? t.trace.succeeded : t.trace.failed}
      </span>
      <span className="text-[11.5px] text-mute">{phaseLabel}</span>
      <span className="text-ghost text-[11px]">·</span>
      <span className="text-[11px] text-ghost">{sourceLabel}</span>
      <span className="text-ghost text-[11px]">·</span>
      <Ltr className="text-[11px] text-ghost font-mono">{trace.duration_ms}ms</Ltr>
    </div>
  );
}

function ValueRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex justify-between gap-3 text-[12px]">
      <span className="text-faint">{label}</span>
      <Ltr className="font-mono text-dim">{value}</Ltr>
    </div>
  );
}

function TraceRow({ trace }: { trace: ZkTrace }) {
  const t = useT();
  const witnessValues = trace.witness_input?.input_data?.[0];
  const circuitValues = trace.circuit_public_values;
  const hasKnownShape = circuitValues && circuitValues.length >= 7;

  return (
    <div className="px-5 py-4 border-b border-border-soft last:border-0">
      <div className="flex items-center justify-between gap-3 mb-2.5">
        <SourceBadge trace={trace} />
        <Ltr className="text-[11px] text-ghost font-mono shrink-0">
          {formatRiyadhStamp(trace.created_at)}
        </Ltr>
      </div>

      {(witnessValues || hasKnownShape) && (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-2.5">
          {witnessValues && (
            <div className="rounded-md border border-border-soft bg-canvas px-3.5 py-3">
              <div className="text-[10.5px] font-medium text-ghost uppercase tracking-wide mb-2">
                {t.trace.witnessInput}
              </div>
              <div className="flex flex-col gap-1">
                {FEATURE_LABELS.map((label, i) => (
                  <ValueRow key={label} label={t.demo[FEATURE_LABEL_KEYS[label]]} value={witnessValues[i]} />
                ))}
              </div>
            </div>
          )}

          {hasKnownShape && (
            <div className="rounded-md border border-border-soft bg-canvas px-3.5 py-3">
              <div className="text-[10.5px] font-medium text-ghost uppercase tracking-wide mb-2">
                {t.trace.circuitOutput}
              </div>
              <div className="flex items-center justify-between gap-3 text-[12px]">
                <span className="text-faint">{t.trace.circuitClass}</span>
                <span
                  className={`inline-block px-2 py-[3px] rounded text-[10.5px] font-medium tracking-[0.03em] uppercase border ${
                    circuitValues[4] === 1
                      ? "text-verified bg-verified/10 border-verified/30"
                      : "text-danger bg-danger/10 border-danger/30"
                  }`}
                >
                  {circuitValues[4] === 1 ? t.demo.approved : t.demo.declined}
                </span>
              </div>
              <div className="mt-2 pt-2 border-t border-border-soft text-[10.5px] text-ghost leading-relaxed">
                {t.trace.driftNote}
              </div>
            </div>
          )}
        </div>
      )}

      {circuitValues && !hasKnownShape && (
        <div className="mb-2.5 text-[11.5px] font-mono text-faint break-all">
          {JSON.stringify(circuitValues)}
        </div>
      )}

      {trace.error_message && (
        <details className="rounded-md border border-danger/30 bg-danger/5 mt-1">
          <summary className="cursor-pointer select-none px-3.5 py-2 text-[11px] font-medium text-danger">
            {t.trace.errorDetail}
          </summary>
          <div className="border-t border-danger/20 px-3.5 py-2.5">
            <pre className="whitespace-pre-wrap break-all font-mono text-[11px] text-faint m-0">
              {trace.error_message}
            </pre>
          </div>
        </details>
      )}

      {!trace.error_message && trace.stdout_tail && (
        <details className="rounded-md border border-border-soft bg-surface-alt mt-1">
          <summary className="cursor-pointer select-none px-3.5 py-2 text-[11px] font-medium text-faint">
            {t.trace.rawLog}
          </summary>
          <div className="border-t border-border-soft px-3.5 py-2.5">
            <pre className="whitespace-pre-wrap break-all font-mono text-[11px] text-faint m-0">
              {trace.stdout_tail}
            </pre>
          </div>
        </details>
      )}
    </div>
  );
}

export function ZkTraceView({ traces }: { traces: ZkTrace[] }) {
  const t = useT();

  if (traces.length === 0) {
    return <div className="px-5 py-6 text-center text-ghost text-[12.5px]">{t.trace.empty}</div>;
  }

  return (
    <div className="rounded-lg border border-border bg-surface overflow-hidden">
      {traces.map((trace) => (
        <TraceRow key={trace.trace_id} trace={trace} />
      ))}
    </div>
  );
}
