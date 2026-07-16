import Link from "next/link";
import { getReceipt, verifyReceipt, getTrace } from "@/lib/api";
import { Ltr } from "@/components/Ltr";
import { SarIcon } from "@/components/SarIcon";
import { VerifyPanel } from "@/components/VerifyPanel";
import { ZkTraceView } from "@/components/ZkTraceView";
import { getDict } from "@/lib/i18n/server";
import { formatRiyadhStamp } from "@/lib/i18n/format";
import { statusForScore, statusClassName, statusLabel } from "@/lib/status";

type Props = {
  params: Promise<{ id: string }>;
};

function Row({ label, value, striped = false }: { label: string; value: React.ReactNode; striped?: boolean }) {
  return (
    <div className={`flex items-start gap-4 px-5 py-2 ${striped ? "bg-surface" : ""}`}>
      <div className="text-[11.5px] text-ghost w-[150px] shrink-0 pt-px">{label}</div>
      <div className="text-[12px] text-faint break-all">{value}</div>
    </div>
  );
}

function MonoRow({ label, value, striped = false }: { label: string; value: string; striped?: boolean }) {
  return (
    <Row
      label={label}
      striped={striped}
      value={<Ltr className="font-mono text-[11.5px] text-faint block">{value}</Ltr>}
    />
  );
}

export default async function ReceiptDetail({ params }: Props) {
  const { id } = await params;
  const { dict: t } = await getDict();

  const [receipt, verification] = await Promise.all([
    getReceipt(id).catch(() => null),
    verifyReceipt(id).catch(() => null),
  ]);
  const trace = receipt ? await getTrace(receipt.decision_id).catch(() => null) : null;

  if (!receipt) {
    return (
      <div>
        <Link href="/audit" className="inline-block mb-4.5 text-[12.5px] text-faint hover:text-ink transition-colors">
          {t.receipt.back}
        </Link>
        <div className="py-16 text-center text-ghost text-sm">{t.receipt.notFound}</div>
      </div>
    );
  }

  const valid = verification?.signature_valid ?? false;
  const hasInput = receipt.input && Object.keys(receipt.input).length > 0;
  const hasOutput = receipt.output && Object.keys(receipt.output).length > 0;
  const status = hasOutput && typeof receipt.output.score === "number" ? statusForScore(receipt.output.score) : null;

  return (
    <div className="max-w-[920px]">
      <Link href="/audit" className="inline-block mb-4.5 text-[12.5px] text-faint hover:text-ink transition-colors">
        {t.receipt.back}
      </Link>

      <div className="flex items-start justify-between mb-7 gap-4">
        <div>
          <h1 className="m-0 mb-1.5 text-[20px] font-semibold text-ink">{t.receipt.title}</h1>
          <Ltr className="font-mono text-[12.5px] text-faint block mb-1">{receipt.receipt_id}</Ltr>
          {receipt.model_name && (
            <div className="text-[13px] text-mute">
              {t.receipt.signedBy(receipt.model_name, receipt.model_version ?? "")}
            </div>
          )}
        </div>
        <div className="flex flex-col items-end gap-2 shrink-0">
          <div
            className={`flex items-center gap-2 px-4 py-2 rounded-md text-[13px] font-semibold border ${
              valid
                ? "text-verified bg-verified/10 border-verified/35"
                : "text-danger bg-danger/10 border-danger/35"
            }`}
          >
            <span className={`w-[7px] h-[7px] rounded-full ${valid ? "bg-verified" : "bg-danger"}`} />
            {valid ? t.receipt.signatureValid : t.receipt.signatureInvalid}
          </div>
          {status && (
            <span
              className={`inline-block px-[11px] py-[5px] rounded text-[11px] font-medium tracking-[0.03em] uppercase border ${statusClassName(status)}`}
            >
              {statusLabel(status, t)}
            </span>
          )}
        </div>
      </div>

      {(hasInput || hasOutput) && (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-5">
          {hasInput && (
            <div className="rounded-lg border border-border bg-surface p-5">
              <div className="text-[13px] font-semibold text-ink mb-3.5">{t.demo.application}</div>
              <div className="flex flex-col gap-2.5 text-[12.5px]">
                {typeof receipt.input.income === "number" && (
                  <div className="flex justify-between">
                    <span className="text-faint">{t.demo.incomeLabel}</span>
                    <span className="inline-flex items-center gap-1 font-mono text-dim">
                      <Ltr>{receipt.input.income.toLocaleString("en-US")}</Ltr>
                      <SarIcon size={11} />
                    </span>
                  </div>
                )}
                {typeof receipt.input.debt_ratio === "number" && (
                  <div className="flex justify-between">
                    <span className="text-faint">{t.demo.debtRatioFieldLabel}</span>
                    <Ltr className="font-mono text-dim">{Math.round(receipt.input.debt_ratio * 100)}%</Ltr>
                  </div>
                )}
                {typeof receipt.input.missed_payments === "number" && (
                  <div className="flex justify-between">
                    <span className="text-faint">{t.demo.missedLabel}</span>
                    <Ltr className="font-mono text-dim">{receipt.input.missed_payments}</Ltr>
                  </div>
                )}
                {typeof receipt.input.credit_history_months === "number" && (
                  <div className="flex justify-between">
                    <span className="text-faint">{t.demo.historyLabel}</span>
                    <Ltr className="font-mono text-dim">{receipt.input.credit_history_months}</Ltr>
                  </div>
                )}
              </div>
            </div>
          )}

          {hasOutput && (
            <div className="rounded-lg border border-border bg-surface p-5">
              <div className="text-[13px] font-semibold text-ink mb-3.5">{t.receipt.outputSection}</div>
              <div className="flex flex-col gap-2.5 text-[12.5px]">
                {typeof receipt.output.score === "number" && (
                  <div className="flex justify-between">
                    <span className="text-faint">{t.demo.score}</span>
                    <Ltr className="font-mono text-dim">{receipt.output.score}</Ltr>
                  </div>
                )}
                {typeof receipt.output.prob_good === "number" && (
                  <div className="flex justify-between">
                    <span className="text-faint">{t.demo.pGood}</span>
                    <Ltr className="font-mono text-dim">{Math.round(receipt.output.prob_good * 100)}%</Ltr>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      <div className="mb-5">
        <VerifyPanel
          input={receipt.input}
          output={receipt.output}
          inputHash={receipt.input_hash}
          outputHash={receipt.output_hash}
        />
      </div>

      <div className="mb-5">
        <div className="mb-2.5 text-[13px] font-semibold text-ink">{t.trace.title}</div>
        <p className="m-0 mb-2.5 text-[12.5px] leading-relaxed text-mute">{t.trace.subtitle}</p>
        <ZkTraceView traces={trace?.traces ?? []} />
      </div>

      <details className="rounded-lg border border-border-soft bg-surface-alt mb-5">
        <summary className="cursor-pointer select-none px-5 py-2.5 text-[11.5px] font-medium text-faint">
          {t.receipt.technicalDetails}
        </summary>
        <div className="border-t border-border-soft py-1">
          <MonoRow label={t.receipt.receiptId} value={receipt.receipt_id} striped />
          <MonoRow label={t.receipt.decisionId} value={receipt.decision_id} />
          <Row
            label={t.receipt.timestamp}
            striped
            value={
              <>
                <Ltr>{formatRiyadhStamp(receipt.timestamp)}</Ltr> {t.common.riyadhTimeShort}
              </>
            }
          />
          <MonoRow label={t.receipt.modelId} value={receipt.model_id} />
          <MonoRow label={t.receipt.modelHash} value={receipt.model_hash} striped />
          <MonoRow label={t.receipt.policyId} value={receipt.policy_id} />
          <MonoRow label={t.receipt.policyHash} value={receipt.policy_hash} striped />
          <MonoRow label={t.receipt.inputHash} value={receipt.input_hash} />
          <MonoRow label={t.receipt.outputHash} value={receipt.output_hash} striped />
          <Row
            label={t.receipt.ed25519}
            value={<Ltr className="font-mono text-[11.5px] text-faint break-all block">{receipt.signature}</Ltr>}
          />
        </div>
      </details>

      <div className="text-[11.5px] text-ghost">{t.receipt.disclaimer}</div>
    </div>
  );
}
