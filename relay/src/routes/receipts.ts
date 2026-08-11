import type { Env } from "../env.js";
import { isAuthorized } from "../lib/auth.js";
import { BLOB_PREFIX, LEDGER_PREFIX, LEDGER_SCHEMA_VERSION, ledgerKey } from "../lib/kv.js";
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
  const flagsById = new Map<string, ReceiptFlags>();
  await listAllKeys(env, BLOB_PREFIX, (id) => {
    const flags = flagsById.get(id) ?? { hasBlob: false, hasLedger: false };
    flags.hasBlob = true;
    flagsById.set(id, flags);
  });
  await listAllKeys(env, LEDGER_PREFIX, (id) => {
    const flags = flagsById.get(id) ?? { hasBlob: false, hasLedger: false };
    flags.hasLedger = true;
    flagsById.set(id, flags);
  });
  return flagsById;
}

/**
 * Scan a full prefix, FOLLOWING the KV list cursor to completion so a listing
 * beyond one page (KV caps a page at 1000 keys) is never SILENTLY truncated
 * (I3). Pilot scale (~150 expected / 300 max, D6) fits in one page today, but a
 * single unpaginated .list() would drop receipts past 1000 with no signal - the
 * puller must see every receipt to reconcile orphans. The prefix-stripped id of
 * each key is handed to `onId`. Mirrors the cursor loop in test/setup.ts.
 */
async function listAllKeys(
  env: Env,
  prefix: string,
  onId: (id: string) => void,
): Promise<void> {
  let cursor: string | undefined;
  do {
    const listing = await env.INTAKE_BLOBS.list(
      cursor === undefined ? { prefix } : { prefix, cursor },
    );
    for (const key of listing.keys) {
      onId(key.name.slice(prefix.length));
    }
    cursor = listing.list_complete ? undefined : listing.cursor;
  } while (cursor !== undefined);
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
    // The ledger key vanished between the prefix list and this get (TTL expiry
    // or an explicit delete): a benign presence race, NOT corruption - no log.
    return null;
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    // The relay's OWN persisted ledger row is unparseable - genuine corruption.
    // Fail loud (I3): log the CLASS only, never the value/body, then keep the
    // graceful row-downgrade (presence flags preserved, metadata omitted).
    console.error("relay_ledger_unreadable");
    return null;
  }
  if (typeof parsed !== "object" || parsed === null) {
    console.error("relay_ledger_unreadable");
    return null;
  }
  const obj = parsed as Record<string, unknown>;
  if (obj["version"] !== LEDGER_SCHEMA_VERSION) {
    // Schema drift (I7): the row was written under a different ledger schema
    // version. A distinct, DETECTABLE condition - not shape-guessed. Log the
    // class only, then downgrade the row.
    console.error("relay_ledger_version_mismatch");
    return null;
  }
  if (
    typeof obj["size"] !== "number" ||
    typeof obj["arrived_at"] !== "string" ||
    typeof obj["claimed_fingerprint"] !== "string"
  ) {
    // Right schema version but a malformed field set: corruption again (I3).
    console.error("relay_ledger_unreadable");
    return null;
  }
  return {
    size: obj["size"],
    arrivedAt: obj["arrived_at"],
    claimedFingerprint: obj["claimed_fingerprint"],
  };
}
