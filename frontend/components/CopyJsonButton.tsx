"use client";

import { useState } from "react";
import { useT } from "@/lib/i18n/LocaleProvider";

type Props = {
  data: unknown;
  label: string;
  className?: string;
};

/**
 * Copies `data` as pretty-printed JSON — for the receipt detail page, this
 * is the exact shape `POST /verify` on the /verify page expects, so a
 * receipt can be copied here and pasted there for standalone verification.
 */
export function CopyJsonButton({ data, label, className = "" }: Props) {
  const t = useT();
  const [copied, setCopied] = useState(false);

  function copy() {
    navigator.clipboard.writeText(JSON.stringify(data, null, 2)).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  }

  return (
    <button
      onClick={copy}
      className={`inline-flex items-center gap-1.5 px-3.5 py-[7px] rounded-md text-[12.5px] font-medium border border-border bg-surface text-dim hover:text-ink hover:border-faint transition-colors cursor-pointer ${className}`}
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="shrink-0">
        <rect x="9" y="9" width="13" height="13" rx="2" />
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
      </svg>
      {copied ? t.common.copied : label}
    </button>
  );
}
