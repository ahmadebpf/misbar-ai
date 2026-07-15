"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Logo } from "@/components/Logo";
import { LocaleToggle } from "@/components/LocaleToggle";
import { useT } from "@/lib/i18n/LocaleProvider";

export function Nav() {
  const pathname = usePathname();
  const t = useT();

  const tabs = [
    { href: "/console", label: t.nav.dashboard, match: (p: string) => p === "/console" },
    { href: "/audit", label: t.nav.audit, match: (p: string) => p.startsWith("/audit") },
    { href: "/demo", label: t.nav.demo, match: (p: string) => p.startsWith("/demo") },
  ];

  return (
    <nav className="sticky top-0 z-20 bg-canvas border-b border-border">
      <div className="max-w-[1240px] mx-auto flex items-center justify-between h-14 px-6">
        <Link href="/" dir="ltr" className="flex items-center gap-[9px] shrink-0">
          <Logo size={18} />
          <span className="font-mono font-semibold text-[15px] tracking-tight text-ink">
            misbar
          </span>
        </Link>

        <div className="flex items-center gap-6">
          <div className="flex items-center gap-1">
            {tabs.map((tab) => {
              const active = tab.match(pathname);
              return (
                <Link
                  key={tab.href}
                  href={tab.href}
                  className={`px-3 py-2 rounded-md text-[13.5px] font-medium whitespace-nowrap transition-colors ${
                    active ? "bg-surface text-ink" : "text-mute hover:text-ink"
                  }`}
                >
                  {tab.label}
                </Link>
              );
            })}
          </div>

          <div className="flex items-center gap-4 shrink-0">
            <div className="flex items-center gap-2 font-mono text-[11px] text-ghost">
              <span className="w-1.5 h-1.5 rounded-full bg-verified" />
              <span>{t.nav.signerOnline}</span>
            </div>
            <LocaleToggle />
          </div>
        </div>
      </div>
    </nav>
  );
}
