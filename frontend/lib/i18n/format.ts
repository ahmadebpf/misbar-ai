/**
 * Technical values (timestamps, counts) always render with Latin digits in a
 * fixed ISO-like shape, regardless of UI language — consistent with hashes
 * and IDs, which are also always LTR/Latin. Callers append a translated unit
 * label (e.g. dict.common.riyadhTime) outside the forced-LTR span, so it
 * still flows naturally with the surrounding sentence direction.
 */
export function formatRiyadhStamp(iso: string): string {
  const d = new Date(iso);
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: "Asia/Riyadh",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).formatToParts(d);

  const get = (type: string) => parts.find((p) => p.type === type)?.value ?? "";
  return `${get("year")}-${get("month")}-${get("day")} ${get("hour")}:${get("minute")}:${get("second")}`;
}

export function formatCount(n: number): string {
  return n.toLocaleString("en-US");
}
