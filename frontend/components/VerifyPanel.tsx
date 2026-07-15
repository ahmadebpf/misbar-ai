"use client";

import { useEffect, useState } from "react";
import { canonicalizeJSON, sha256Hex } from "@/lib/verify";
import { Ltr } from "@/components/Ltr";
import { useT } from "@/lib/i18n/LocaleProvider";

type Check = { computed: string; matches: boolean } | null;
type Verification = {
  input: unknown;
  output: unknown;
  inputHash: string;
  outputHash: string;
  inputCheck: Check;
  outputCheck: Check;
};

function CheckRow({
  label,
  storedHash,
  check,
}: {
  label: string;
  storedHash: string;
  check: Check;
}) {
  const t = useT();

  return (
    <div className="flex items-start gap-4 px-5 py-3">
      <div className="text-[12px] text-faint w-[170px] shrink-0 pt-px">{label}</div>
      <div className="flex-1 min-w-0">
        {check === null ? (
          <span className="text-[12.5px] text-ghost">{t.receipt.computing}</span>
        ) : (
          <div className="flex flex-col gap-1.5">
            <div className="flex items-baseline gap-2">
              <span className="text-[10.5px] text-ghost w-[78px] shrink-0">{t.receipt.signedLabel}</span>
              <Ltr className="font-mono text-[12px] text-faint break-all block">{storedHash}</Ltr>
            </div>
            <div className="flex items-baseline gap-2">
              <span className="text-[10.5px] text-ghost w-[78px] shrink-0">{t.receipt.recomputedLabel}</span>
              <Ltr className="font-mono text-[12px] text-dim break-all block">{check.computed}</Ltr>
            </div>
            <span
              className={`inline-flex items-center gap-1.5 text-[11.5px] font-medium w-fit ${
                check.matches ? "text-verified" : "text-danger"
              }`}
            >
              <span className={`w-1.5 h-1.5 rounded-full ${check.matches ? "bg-verified" : "bg-danger"}`} />
              {check.matches ? t.receipt.hashMatches : t.receipt.hashMismatch}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

export function VerifyPanel({
  input,
  output,
  inputHash,
  outputHash,
}: {
  input: unknown;
  output: unknown;
  inputHash: string;
  outputHash: string;
}) {
  const t = useT();
  const [verification, setVerification] = useState<Verification | null>(null);
  const isCurrentVerification =
    verification !== null &&
    verification.input === input &&
    verification.output === output &&
    verification.inputHash === inputHash &&
    verification.outputHash === outputHash;
  const inputCheck = isCurrentVerification ? verification.inputCheck : null;
  const outputCheck = isCurrentVerification ? verification.outputCheck : null;
  const done = inputCheck !== null && outputCheck !== null;
  const allMatch = done && inputCheck!.matches && outputCheck!.matches;

  useEffect(() => {
    let cancelled = false;

    (async () => {
      const [computedInput, computedOutput] = await Promise.all([
        sha256Hex(canonicalizeJSON(input)),
        sha256Hex(canonicalizeJSON(output)),
      ]);
      if (cancelled) return;
      setVerification({
        input,
        output,
        inputHash,
        outputHash,
        inputCheck: { computed: computedInput, matches: computedInput === inputHash },
        outputCheck: { computed: computedOutput, matches: computedOutput === outputHash },
      });
    })();

    return () => {
      cancelled = true;
    };
  }, [input, output, inputHash, outputHash]);

  return (
    <div>
      <div className="mb-2.5 text-[13px] font-semibold text-ink">{t.receipt.verifySection}</div>

      <div className="rounded-lg border border-border bg-surface px-5 py-4 mb-2.5">
        <p className="m-0 mb-3 text-[12.5px] leading-relaxed text-mute">{t.receipt.verifyIntro}</p>

        <div
          className={`flex items-center gap-2 px-3.5 py-2.5 rounded-md text-[13px] font-semibold border ${
            !done
              ? "text-ghost bg-surface-alt border-border"
              : allMatch
              ? "text-verified bg-verified/10 border-verified/35"
              : "text-danger bg-danger/10 border-danger/35"
          }`}
        >
          {!done ? (
            <>{t.receipt.computing}</>
          ) : (
            <>
              <span className={`w-[7px] h-[7px] rounded-full ${allMatch ? "bg-verified" : "bg-danger"}`} />
              {allMatch ? t.receipt.verifiedSummary : t.receipt.verifiedSummaryFail}
            </>
          )}
        </div>
      </div>

      <details className="rounded-lg border border-border-soft bg-surface-alt">
        <summary className="cursor-pointer select-none px-5 py-2.5 text-[11.5px] font-medium text-faint">
          {t.receipt.proofDetails}
        </summary>
        <div className="border-t border-border-soft">
          <CheckRow label={t.receipt.inputHash} storedHash={inputHash} check={inputCheck} />
          <div className="h-px bg-border-soft mx-5" />
          <CheckRow label={t.receipt.outputHash} storedHash={outputHash} check={outputCheck} />
        </div>
      </details>
    </div>
  );
}
