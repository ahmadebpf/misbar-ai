/**
 * Mirrors backend/src/execution/features/mod.rs::canonicalize exactly:
 * object keys sorted ascending, no whitespace, recursive. Must stay in sync
 * with the Rust implementation — this is what makes independent client-side
 * hash recomputation actually mean something.
 */
export function canonicalizeJSON(value: unknown): string {
  if (value === null || value === undefined) return "null";
  if (Array.isArray(value)) {
    return `[${value.map(canonicalizeJSON).join(",")}]`;
  }
  if (typeof value === "object") {
    const obj = value as Record<string, unknown>;
    const keys = Object.keys(obj).sort();
    const pairs = keys.map((k) => `${JSON.stringify(k)}:${canonicalizeJSON(obj[k])}`);
    return `{${pairs.join(",")}}`;
  }
  return JSON.stringify(value);
}

export async function sha256Hex(text: string): Promise<string> {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
