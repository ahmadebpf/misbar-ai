"use client";

import { useLocale } from "@/lib/i18n/LocaleProvider";

export function LocaleToggle({ className }: { className?: string }) {
  const { locale, dict, setLocale } = useLocale();

  return (
    <button
      onClick={() => setLocale(locale === "ar" ? "en" : "ar")}
      className={`font-sans text-[11px] text-mute hover:text-ink border border-border rounded px-2 py-1 transition-colors ${className ?? ""}`}
      style={
        locale === "en"
          ? { fontFamily: "var(--font-plex-sans-arabic), var(--font-plex-sans), sans-serif" }
          : undefined
      }
    >
      {dict.common.toggleTo}
    </button>
  );
}
