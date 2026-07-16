import Link from "next/link";
import { Logo } from "@/components/Logo";
import { LocaleToggle } from "@/components/LocaleToggle";
import { Ltr } from "@/components/Ltr";
import { getDict } from "@/lib/i18n/server";
import { formatRiyadhStamp } from "@/lib/i18n/format";

const SAMPLE_RECEIPT_TIME = "2026-07-15T09:12:04Z";

export default async function Landing() {
  const { dict: t } = await getDict();
  const steps = t.landing.steps.map((s, i) => ({ ...s, n: `0${i + 1}`, active: i === 1 }));

  return (
    <div className="min-h-screen bg-canvas text-ink">
      <header className="flex items-center justify-between max-w-[1120px] mx-auto px-6 pt-6">
        <div dir="ltr" className="flex items-center gap-[9px]">
          <Logo size={18} />
          <span className="flex items-center gap-1.5">
            <span className="font-mono font-semibold text-[15px] tracking-tight">Saleem</span>
            <span className="text-ghost text-[13px]">·</span>
            <span
              className="text-[17px]"
              style={{ fontFamily: "var(--font-noto-naskh), var(--font-plex-sans-arabic), sans-serif" }}
            >
              سليم
            </span>
          </span>
        </div>
        <div className="flex items-center gap-3">
          <LocaleToggle />
          <Link
            href="/console"
            className="text-[13px] text-mute border border-border rounded-md px-3.5 py-2 hover:text-ink hover:border-hairline transition-colors"
          >
            {t.nav.launchConsole}
          </Link>
        </div>
      </header>

      <section className="max-w-[920px] mx-auto px-6 pt-24 pb-[72px] text-center">
        <div className="font-mono text-[11.5px] tracking-[0.08em] uppercase text-ghost mb-5">
          {t.landing.kicker}
        </div>
        <h1 className="m-0 mb-[22px] text-[46px] leading-[1.15] font-semibold tracking-[-0.01em]" style={{ textWrap: "balance" }}>
          {t.landing.h1}
        </h1>
        <p className="mx-auto mb-9 max-w-[620px] text-[16px] leading-[1.65] text-mute">
          {t.landing.sub}
        </p>
        <div className="flex items-center justify-center gap-3">
          <Link
            href="/demo"
            className="px-[22px] py-3 rounded-md bg-accent/10 border border-accent text-accent text-[14px] font-semibold hover:bg-accent/20 transition-colors"
          >
            {t.landing.ctaPrimary}
          </Link>
          <Link
            href="/audit"
            className="px-[22px] py-3 rounded-md border border-border text-dim text-[14px] font-medium hover:border-hairline hover:text-ink transition-colors"
          >
            {t.landing.ctaSecondary}
          </Link>
        </div>
      </section>

      <section className="max-w-[640px] mx-auto mb-[88px] px-6">
        <div className="rounded-[10px] border border-border bg-surface px-[26px] py-[22px]">
          <div className="flex items-center justify-between mb-4">
            <Ltr className="font-mono text-[11px] uppercase tracking-[0.04em] text-ghost">
              {t.landing.receiptPrefix} de3d0f2b…
            </Ltr>
            <span className="flex items-center gap-1.5 px-2.5 py-1 rounded text-[11px] font-semibold uppercase tracking-[0.02em] text-verified bg-verified/10 border border-verified/30">
              <span className="w-1.5 h-1.5 rounded-full bg-verified" />
              {t.landing.verified}
            </span>
          </div>
          <div className="flex flex-col gap-[9px] text-[12.5px]">
            <div className="flex justify-between">
              <span className="text-faint">{t.landing.model}</span>
              <Ltr className="font-mono text-dim">credit-risk-v2.3.1</Ltr>
            </div>
            <div className="flex justify-between">
              <span className="text-faint">{t.landing.decision}</span>
              <span className="font-mono text-dim">{t.landing.approvedScore(742)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-faint">{t.landing.signed}</span>
              <span className="font-mono text-dim">
                <Ltr>{formatRiyadhStamp(SAMPLE_RECEIPT_TIME)}</Ltr> {t.common.riyadhTime}
              </span>
            </div>
          </div>
        </div>
      </section>

      <section className="max-w-[800px] mx-auto mb-24 px-6 text-center">
        <h2 className="m-0 mb-[18px] text-2xl font-semibold">{t.landing.problemTitle}</h2>
        <p className="m-0 mb-4 text-[15px] leading-[1.7] text-mute">{t.landing.problemP1}</p>
        <p className="m-0 text-[15px] leading-[1.7] text-mute">{t.landing.problemP2}</p>
      </section>

      <section className="max-w-[1120px] mx-auto mb-[100px] px-6">
        <h2 className="m-0 mb-10 text-[13px] font-semibold tracking-[0.04em] uppercase text-faint text-center">
          {t.landing.howItWorks}
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-10 sm:gap-0 relative">
          <div className="hidden sm:block absolute top-[19px] left-[16.6%] right-[16.6%] h-px bg-border" />
          {steps.map((s) => (
            <div key={s.n} className="text-center px-7 relative">
              <div
                className={`w-[38px] h-[38px] mx-auto mb-5 rounded-full bg-canvas border flex items-center justify-center font-mono text-[13px] relative z-10 ${
                  s.active ? "border-accent text-accent" : "border-hairline text-dim"
                }`}
              >
                <Ltr>{s.n}</Ltr>
              </div>
              <div className="text-[14.5px] font-semibold mb-2">{s.title}</div>
              <div className="text-[13px] leading-[1.6] text-faint">{s.body}</div>
            </div>
          ))}
        </div>
      </section>

      <section className="border-t border-border px-6 pt-14 pb-[72px] text-center">
        <h2 className="m-0 mb-3 text-[22px] font-semibold">{t.landing.finalTitle}</h2>
        <p className="m-0 mb-[26px] text-sm text-faint">{t.landing.finalSub}</p>
        <Link
          href="/demo"
          className="inline-block px-[26px] py-[13px] rounded-md bg-accent/10 border border-accent text-accent text-[14px] font-semibold hover:bg-accent/20 transition-colors"
        >
          {t.landing.finalCta}
        </Link>
      </section>
    </div>
  );
}
