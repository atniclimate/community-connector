import type { Env } from "../env.js";
import { isAuthorized } from "../lib/auth.js";
import { BLOB_PREFIX, LEDGER_PREFIX, ledgerKey } from "../lib/kv.js";
import { jsonResponse, notFound } from "../lib/http.js";

interface ReceiptFlags {
  hasBlob: boolean;
  hasLedger: boolean;
}

/**
 * GET /receipts (authenticated control plane). Lists the UNION of the blob: and
 * ledger: prefixes and returns a merged presence view, so the puller learns
 * from ONE call both "ledger exists but no blob" (normal: pulled/expired) and
 * "blob exists but no ledger" (the orphan anomaly from a ledger-write crash) -
 * this is what makes D6's "the puller lists BOTH prefixes" true given the
 * blueprint defines only this one listing route. Never surfaces blob CONTENT,
 * only presence (D1: the listing must not become a second ciphertext leak).
 *
 * Auth failure returns the plain 404 (D6 401-vs-404 rule), NOT an empty
 * {"receipts":[]} - an empty typed body would itself confirm the token was
 * recognized as a token.
 */
export async function handleReceipts(request: Request, env: Env): Promise<Response> {
  if (!(await isAuthorized(request, env))) {
    return notFound();
  }

  const flagsById = await collectPresence(env);
  const receipts = await Promise.all(
    [...flagsById.entries()].map(([receiptId, flags]) => buildReceipt(env, receiptId, flags)),
  );

  // Pilot-scale simplification (~150 expected / 300 max, D6): one full .list()
  // per prefix, merged in memory, cursor always null. The cursor field exists
  // for forward compatibility; real cross-prefix cursor merging is meaningfully
  // more complex than this scale justifies.
  return jsonResponse(200, { receipts, cursor: null });
}

/** Union the two prefixes into per-receipt presence flags. */
async function collectPresence(env: Env): Promise<Map<string, ReceiptFlags>> {
  const [blobList, ledgerList] = await Promise.all([
    env.INTAKE_BLOBS.list({ prefix: BLOB_PREFIX }),
    env.INTAKE_BLOBS.list({ prefix: LEDGER_PREFIX }),
  ]);

  const flagsById = new Map<string, ReceiptFlags>();
  for (const key of blobList.keys) {
    const id = key.name.slice(BLOB_PREFIX.length);
    const flags = flagsById.get(id) ?? { hasBlob: false, hasLedger: false };
    flags.hasBlob = true;
    flagsById.set(id, flags);
  }
  for (const key of ledgerList.keys) {
    const id = key.name.slice(LEDGER_PREFIX.length);
    const flags = flagsById.get(id) ?? { hasBlob: false, hasLedger: false };
    flags.hasLedger = true;
    flagsById.set(id, flags);
  }
  return flagsById;
}

/**
 * One receipt row. Metadata fields (size, arrived_at, claimed_fingerprint) are
 * present only when has_ledger is true and the ledger value reads and parses;
 * flags are always independent (never assume the two prefixes agree).
 */
async function buildReceipt(
  env: Env,
  receiptId: string,
  flags: ReceiptFlags,
): Promise<Record<string, unknown>> {
  const base = {
    receipt_id: receiptId,
    has_ledger: flags.hasLedger,
    has_blob: flags.hasBlob,
  };
  if (!flags.hasLedger) {
    return base;
  }
  const meta = await readLedger(env, receiptId);
  if (meta === null) {
    return base;
  }
  return { ...base, size: meta.size, arrived_at: meta.arrivedAt, claimed_fingerprint: meta.claimedFingerprint };
}

interface LedgerMeta {
  size: number;
  arrivedAt: string;
  claimedFingerprint: string;
}

/** Read and type-check a ledger entry; null if missing or malformed. */
async function readLedger(env: Env, receiptId: string): Promise<LedgerMeta | null> {
  const raw = await env.INTAKE_BLOBS.get(ledgerKey(receiptId));
  if (raw === null) {
    return null;
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return null;
  }
  if (typeof parsed !== "object" || parsed === null) {
    return null;
  }
  const obj = parsed as Record<string, unknown>;
  if (
    typeof obj["size"] !== "number" ||
    typeof obj["arrived_at"] !== "string" ||
    typeof obj["claimed_fingerprint"] !== "string"
  ) {
    return null;
  }
  return {
    size: obj["size"],
    arrivedAt: obj["arrived_at"],
    claimedFingerprint: obj["claimed_fingerprint"],
  };
}
