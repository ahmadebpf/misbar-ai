"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import {
  verifyReceipt,
  verifyExternal,
  getTrace,
  listTraces,
  type VerificationPackage,
  type ZkTrace,
} from "@/lib/api";
import { HashDisplay } from "@/components/HashDisplay";
import { Ltr } from "@/components/Ltr";
import { ZkTraceView } from "@/components/ZkTraceView";
import { useLocale } from "@/lib/i18n/LocaleProvider";
import { formatRiyadhStamp } from "@/lib/i18n/format";

type VerifyOutcome = { pkg: VerificationPackage; traces: ZkTrace[] };

function ResultCard({ outcome }: { outcome: VerifyOutcome }) {
  const { dict: t } = useLocale();
  const { pkg, traces } = outcome;

  return (
    <div className="mt-4">
      <div className="flex flex-wrap items-center gap-2 mb-3">
        <span
          className={`inline-flex items-center gap-2 px-3.5 py-2 rounded-md text-[13px] font-semibold border ${
            pkg.verified ? "text-verified bg-verified/10 border-verified/35" : "text-danger bg-danger/10 border-danger/35"
          }`}
        >
          <span className={`w-[7px] h-[7px] rounded-full ${pkg.verified ? "bg-verified" : "bg-danger"}`} />
          {pkg.verified ? t.receipt.verified : t.receipt.notVerified}
        </span>
        <span className="text-[11px] text-ghost">
          {t.common.signatureLabel}: {pkg.signature_valid ? "✓" : "✗"} · {t.common.proofLabel}: {pkg.proof_valid ? "✓" : "✗"}
        </span>
      </div>

      <div className="flex items-center gap-3 mb-4 text-[12.5px]">
        <HashDisplay value={pkg.receipt.receipt_id} chars={8} label={t.receipt.receiptId} />
        <Link href={`/audit/${pkg.receipt.receipt_id}`} className="text-accent hover:text-accent-soft font-medium">
          {t.verify.viewReceipt}
        </Link>
      </div>

      <div className="mb-1.5 text-[12px] font-semibold text-ink">{t.trace.title}</div>
      <ZkTraceView traces={traces} />
    </div>
  );
}

function LookupForm() {
  const { dict: t, locale } = useLocale();
  const [id, setId] = useState("");
  const [loading, setLoading] = useState(false);
  const [outcome, setOutcome] = useState<VerifyOutcome | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    if (!id.trim()) return;
    setLoading(true);
    setError(null);
    setOutcome(null);
    try {
      const pkg = await verifyReceipt(id.trim());
      const traceResp = await getTrace(pkg.receipt.decision_id).catch(() => ({ decision_id: "", traces: [] as ZkTrace[] }));
      setOutcome({ pkg, traces: traceResp.traces });
    } catch {
      setError(t.verify.notFound);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="rounded-lg border border-border bg-surface p-6">
      <div className="text-[13px] font-semibold text-ink mb-3.5">{t.verify.lookupTitle}</div>
      <div className="flex gap-2.5">
        <input
          dir="ltr"
          value={id}
          onChange={(e) => setId(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          placeholder={t.verify.lookupPlaceholder}
          className={`flex-1 min-w-0 bg-surface-alt border border-border rounded-md px-3 py-2.5 font-mono text-[12.5px] text-ink outline-none focus:border-hairline ${
            locale === "ar" ? "placeholder-arabic" : ""
          }`}
        />
        <button
          onClick={submit}
          disabled={loading || !id.trim()}
          className="px-4 py-2.5 rounded-md border border-accent bg-accent/10 text-accent text-[13px] font-semibold cursor-pointer hover:bg-accent/20 disabled:opacity-40 disabled:cursor-not-allowed transition-colors shrink-0"
        >
          {loading ? t.verify.verifying : t.verify.lookupSubmit}
        </button>
      </div>

      {error && (
        <div className="mt-3.5 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-[12.5px] text-danger">
          {error}
        </div>
      )}

      {outcome && <ResultCard outcome={outcome} />}
    </div>
  );
}

function PasteForm() {
  const { dict: t, locale } = useLocale();
  const [text, setText] = useState("");
  const [loading, setLoading] = useState(false);
  const [outcome, setOutcome] = useState<VerifyOutcome | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    setLoading(true);
    setError(null);
    setOutcome(null);

    let parsed: unknown;
    try {
      parsed = JSON.parse(text);
    } catch {
      setError(t.verify.invalidJson);
      setLoading(false);
      return;
    }

    try {
      const pkg = await verifyExternal(parsed);
      const traceResp = await getTrace(pkg.receipt.decision_id).catch(() => ({ decision_id: "", traces: [] as ZkTrace[] }));
      setOutcome({ pkg, traces: traceResp.traces });
    } catch (e) {
      setError(e instanceof Error ? e.message : t.verify.invalidJson);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="rounded-lg border border-border bg-surface p-6">
      <div className="text-[13px] font-semibold text-ink mb-1.5">{t.verify.pasteTitle}</div>
      <p className="m-0 mb-3.5 text-[12px] text-faint leading-relaxed">{t.verify.pasteSubtitle}</p>
      <textarea
        dir="ltr"
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder={t.verify.pastePlaceholder}
        rows={8}
        className={`w-full box-border bg-surface-alt border border-border rounded-md px-3 py-2.5 font-mono text-[11.5px] text-ink outline-none focus:border-hairline resize-y ${
          locale === "ar" ? "placeholder-arabic" : ""
        }`}
      />
      <button
        onClick={submit}
        disabled={loading || !text.trim()}
        className="mt-2.5 px-4 py-2.5 rounded-md border border-accent bg-accent/10 text-accent text-[13px] font-semibold cursor-pointer hover:bg-accent/20 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
      >
        {loading ? t.verify.verifying : t.verify.pasteSubmit}
      </button>

      {error && (
        <div className="mt-3.5 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-[12.5px] text-danger">
          {error}
        </div>
      )}

      {outcome && <ResultCard outcome={outcome} />}
    </div>
  );
}

function RecentActivity() {
  const { dict: t } = useLocale();
  const [traces, setTraces] = useState<ZkTrace[] | null>(null);

  useEffect(() => {
    listTraces(10, 0)
      .then((r) => setTraces(r.traces))
      .catch(() => setTraces([]));
  }, []);

  return (
    <div className="mt-7">
      <div className="mb-2.5 text-[13px] font-semibold text-ink">{t.verify.recentActivity}</div>
      <div className="rounded-lg border border-border bg-surface overflow-hidden">
        {!traces ? (
          <div className="px-5 py-8 text-center text-ghost text-[12.5px]">…</div>
        ) : traces.length === 0 ? (
          <div className="px-5 py-8 text-center text-ghost text-[12.5px]">{t.verify.recentEmpty}</div>
        ) : (
          traces.map((trace) => (
            <div
              key={trace.trace_id}
              className="flex items-center gap-4 px-5 py-3 text-[12.5px] border-b border-border-soft last:border-0"
            >
              <span
                className={`inline-flex items-center gap-1.5 px-2 py-[3px] rounded text-[10.5px] font-medium border shrink-0 ${
                  trace.success ? "text-verified bg-verified/10 border-verified/35" : "text-danger bg-danger/10 border-danger/35"
                }`}
              >
                <span className={`w-[5px] h-[5px] rounded-full ${trace.success ? "bg-verified" : "bg-danger"}`} />
                {trace.phase === "prove" ? t.trace.phaseProve : t.trace.phaseVerify}
              </span>
              <HashDisplay value={trace.decision_id} chars={6} />
              <span className="text-ghost text-[11px] shrink-0">
                {trace.source === "gateway" ? t.trace.sourceGateway : trace.source === "lookup" ? t.trace.sourceLookup : t.trace.sourcePasted}
              </span>
              <span className="flex-1" />
              <Ltr className="font-mono text-[11px] text-faint shrink-0">{formatRiyadhStamp(trace.created_at)}</Ltr>
              {trace.receipt_id && (
                <Link href={`/audit/${trace.receipt_id}`} className="text-accent hover:text-accent-soft shrink-0">
                  {t.verify.viewReceipt}
                </Link>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

export default function VerifyPage() {
  const { dict: t } = useLocale();

  return (
    <div className="max-w-[820px]">
      <div className="mb-7">
        <h1 className="m-0 mb-1.5 text-[22px] font-semibold text-ink">{t.verify.title}</h1>
        <p className="m-0 text-[13.5px] text-faint">{t.verify.subtitle}</p>
      </div>

      <div className="flex flex-col gap-5">
        <LookupForm />
        <PasteForm />
      </div>

      <RecentActivity />
    </div>
  );
}
