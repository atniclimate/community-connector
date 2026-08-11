/**
 * Worker bindings (ADR-005 D6; blueprint intake-relay section 5.3). Vars are
 * KV/wrangler string values; both secrets are set via `wrangler secret put` at
 * deploy time and are never in the repo.
 */
export interface Env {
  /** One KV namespace, two key prefixes: blob:<id> and ledger:<id>. */
  INTAKE_BLOBS: KVNamespace;

  // --- [vars] (deploy-runbook configuration, D6) ---
  /** Blob TTL in seconds (>= 2x max pull interval, <= pilot window). */
  BLOB_TTL_SECONDS: string;
  /** Ledger TTL in seconds (blob TTL + consistency margin + >= one pull interval). */
  LEDGER_TTL_SECONDS: string;
  /** Per-blob size cap in bytes (single-digit KB, D6). */
  MAX_BLOB_SIZE_BYTES: string;
  /** Approximate total-blob cap (eventually-consistent list count, not a hard cap). */
  APPROXIMATE_BLOB_CAP: string;
  /** Generous per-IP fixed-window rate limit (NAT-safe venue scale, D6). */
  RATE_LIMIT_PER_IP_PER_MINUTE: string;
  /** Exact Pages origin allowed for CORS on POST /submit (never "*"). */
  PAGES_ORIGIN: string;

  // --- secrets (wrangler secret put; never committed) ---
  /** SHA-256 hex of the control-plane bearer token (verifier model, D6). */
  CREDENTIAL_HASH: string;
  /** Comma-separated admitted key fingerprints (the D3/D6 rotation cutoff). */
  ADMISSION_ALLOWLIST: string;
}
