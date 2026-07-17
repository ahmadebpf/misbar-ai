import type { Dict } from "@/lib/i18n/dictionaries";

export type DecisionStatus = "approved" | "review" | "declined";

export function statusForScore(score: number): DecisionStatus {
  if (score >= 700) return "approved";
  if (score >= 580) return "review";
  return "declined";
}

export function statusClassName(status: DecisionStatus): string {
  switch (status) {
    case "approved":
      return "text-verified bg-verified/10 border-verified/30";
    case "review":
      return "text-pending bg-pending/10 border-pending/30";
    case "declined":
      return "text-danger bg-danger/10 border-danger/30";
  }
}

export function statusLabel(status: DecisionStatus, t: Dict): string {
  return t.demo[status];
}
