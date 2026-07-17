const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3000";

// The gateway treats model input/output as opaque JSON — these known fields
// are specific to the credit-decision demo model; unknown fields pass through.
export type DecisionInput = {
  income?: number;
  debt_ratio?: number;
  missed_payments?: number;
  credit_history_months?: number;
  [key: string]: unknown;
};

export type DecisionOutput = {
  score?: number;
  prob_good?: number;
  [key: string]: unknown;
};

export type Receipt = {
  receipt_id: string;
  decision_id: string;
  model_id: string;
  model_hash: string;
  model_name?: string | null;
  model_version?: string | null;
  policy_id: string;
  policy_hash: string;
  policy_name?: string | null;
  policy_version?: string | null;
  input: DecisionInput;
  input_hash: string;
  output: DecisionOutput;
  output_hash: string;
  timestamp: string;
  signature: string;
};

export type Stats = {
  total_receipts: number;
  model_id: string;
  model_hash: string;
  model_name?: string | null;
  model_version?: string | null;
  policy_id: string;
  policy_hash: string;
  policy_name?: string | null;
  policy_version?: string | null;
};

export type ReceiptsResponse = {
  receipts: Receipt[];
  total: number;
  limit: number;
  offset: number;
};

export type VerificationPackage = {
  receipt: Receipt;
  verified: boolean;
  signature_valid: boolean;
  proof_valid: boolean;
};

// Mirrors backend/src/domain.rs::ZkTrace — one row per ezkl attempt
// (prove or verify), success or failure, linked by decision_id so failed
// proves are visible even though they never produce a receipt.
export type ZkTrace = {
  trace_id: string;
  decision_id: string;
  receipt_id: string | null;
  phase: "prove" | "verify";
  source: "gateway" | "lookup" | "pasted";
  success: boolean;
  witness_input: { input_data: number[][] } | null;
  circuit_public_values: number[] | null;
  error_message: string | null;
  stdout_tail: string | null;
  duration_ms: number;
  created_at: string;
};

export type TraceResponse = { decision_id: string; traces: ZkTrace[] };
export type TracesResponse = { traces: ZkTrace[]; limit: number; offset: number };

async function apiFetch<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, { cache: "no-store" });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

async function apiPost<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
    cache: "no-store",
  });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

export const getStats = () => apiFetch<Stats>("/stats");

export const listReceipts = (limit = 20, offset = 0) =>
  apiFetch<ReceiptsResponse>(`/receipts?limit=${limit}&offset=${offset}`);

export const getReceipt = (id: string) =>
  apiFetch<Receipt>(`/receipt/${id}`);

export const verifyReceipt = (id: string) =>
  apiFetch<VerificationPackage>(`/verify/${id}`);

// Standalone verification — the caller pastes a full receipt (same shape
// getReceipt returns) and it's re-verified without a store lookup.
export const verifyExternal = (receipt: unknown) =>
  apiPost<VerificationPackage>("/verify", receipt);

export const getTrace = (decisionId: string) =>
  apiFetch<TraceResponse>(`/trace/${decisionId}`);

export const listTraces = (limit = 20, offset = 0) =>
  apiFetch<TracesResponse>(`/traces?limit=${limit}&offset=${offset}`);
