"use client";

import { useState } from "react";
import { useT } from "@/lib/i18n/LocaleProvider";
import { Ltr } from "@/components/Ltr";

type Props = {
  value: string;
  chars?: number;
  label?: string;
};

export function HashDisplay({ value, chars = 8, label }: Props) {
  const t = useT();
  const [copied, setCopied] = useState(false);
  const safeValue = value ?? "";

  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    e.preventDefault();
    navigator.clipboard.writeText(safeValue).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const display =
    safeValue.length > chars * 2 + 3
      ? `${safeValue.slice(0, chars)}…${safeValue.slice(-chars)}`
      : safeValue;

  return (
    <span className="inline-flex items-center gap-1.5 font-mono">
      {label && <span className="text-[11px] text-ghost me-0.5">{label}</span>}
      <Ltr
        onClick={copy}
        title={safeValue}
        className="text-[12.5px] text-dim cursor-pointer border-b border-dashed border-hairline hover:text-ink hover:border-faint pb-px transition-colors"
      >
        {display}
      </Ltr>
      {copied ? (
        <span className="text-[10.5px] font-medium text-verified">{t.common.copied}</span>
      ) : (
        <svg
          onClick={copy}
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          className="cursor-pointer shrink-0 text-ghost hover:text-ink transition-colors"
        >
          <rect x="9" y="9" width="13" height="13" rx="2" />
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
      )}
    </span>
  );
}
