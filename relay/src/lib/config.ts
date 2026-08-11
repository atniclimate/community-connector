import type { Env } from "../env.js";

/**
 * Parsed numeric configuration (ADR-005 D6; blueprint 5.3). Vars arrive as
 * strings from wrangler/KV; this is the one place they become numbers, with a
 * loud fail on a nonsense value (a misconfigured deploy fails closed as a
 * generic 500 rather than silently treating an unparseable cap as 0 or NaN).
 */
export interface RelayConfig {
  blobTtlSeconds: number;
  ledgerTtlSeconds: number;
  maxBlobSizeBytes: number;
  approximateBlobCap: number;
  rateLimitPerIpPerMinute: number;
}

export function readConfig(env: Env): RelayConfig {
  return {
    blobTtlSeconds: positiveInt(env.BLOB_TTL_SECONDS, "BLOB_TTL_SECONDS"),
    ledgerTtlSeconds: positiveInt(env.LEDGER_TTL_SECONDS, "LEDGER_TTL_SECONDS"),
    maxBlobSizeBytes: positiveInt(env.MAX_BLOB_SIZE_BYTES, "MAX_BLOB_SIZE_BYTES"),
    approximateBlobCap: positiveInt(env.APPROXIMATE_BLOB_CAP, "APPROXIMATE_BLOB_CAP"),
    rateLimitPerIpPerMinute: positiveInt(
      env.RATE_LIMIT_PER_IP_PER_MINUTE,
      "RATE_LIMIT_PER_IP_PER_MINUTE",
    ),
  };
}

function positiveInt(value: string | undefined, name: string): number {
  const n = Number(value);
  if (!Number.isInteger(n) || n <= 0) {
    // The name is a static config identifier, never user input; the top-level
    // handler logs only the error class and returns a generic 500, so this
    // never reaches a client body.
    throw new Error(`invalid config: ${name}`);
  }
  return n;
}
