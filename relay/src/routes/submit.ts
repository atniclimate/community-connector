import type { Env } from "../env.js";
import { readConfig, type RelayConfig } from "../lib/config.js";
import { submitResponse } from "../lib/cors.js";
import { toHex } from "../lib/hex.js";
import { blobKey, ledgerKey } from "../lib/kv.js";
import { BLOB_PREFIX } from "../lib/kv.js";
import { errorName } from "../lib/http.js";
import { checkRateLimit } from "../lib/ratelimit.js";

/** Fixed Retry-After for 429/503 (exact seconds not load-bearing at this scale). */
const RETRY_AFTER_SECONDS = "60";

/**
 * POST /submit (public, no auth): accept exactly one outer envelope as opaque
 * ciphertext (ADR-005 D1/D6). Ordered validation, cheapest checks first, fail
 * fast. EVERY response - success and every error - carries the /submit CORS
 * origin header via submitResponse (D5). The relay never decrypts, never
 * inspects the ciphertext, and stores the body byte-for-byte verbatim.
 */
export async function handleSubmit(request: Request, env: Env): Promise<Response> {
  let config: RelayConfig;
  try {
    config = readConfig(env);
  } catch (err) {
    console.error("relay_config_error", errorName(err));
    return submitResponse(env, 500, { error: "internal_error" });
  }

  // 1. Cheap Content-Length pre-check (a client can lie about or omit it, so
  //    this is not authoritative - the byte check in step 3 is).
  const contentLength = request.headers.get("Content-Length");
  if (contentLength !== null) {
    const declared = Number(contentLength);
    if (Number.isFinite(declared) && declared > config.maxBlobSizeBytes) {
      return submitResponse(env, 413, { error: "payload_too_large" });
    }
  }

  // 2. Content-Type must be application/json (charset param tolerated).
  const mediaType = (request.headers.get("Content-Type") ?? "").split(";")[0]?.trim().toLowerCase();
  if (mediaType !== "application/json") {
    return submitResponse(env, 400, { error: "invalid_content_type" });
  }

  // 3. Read raw bytes and check the EXACT byte length (not a decoded string's
  //    .length, which counts UTF-16 units and under-counts multi-byte UTF-8).
  const bytes = new Uint8Array(await request.arrayBuffer());
  if (bytes.byteLength > config.maxBlobSizeBytes) {
    return submitResponse(env, 413, { error: "payload_too_large" });
  }
  const rawBody = new TextDecoder("utf-8").decode(bytes);

  // 4-6. Structural validation of the cleartext routing fields only.
  const fields = parseEnvelopeFields(rawBody);
  if (fields === null) {
    return submitResponse(env, 400, { error: "invalid_envelope" });
  }
  // 7. intake_envelope_version is DELIBERATELY not checked: unknown-major
  //    rejection for it is the puller's job (D3), never the relay's.

  // 8. Admission allowlist (the D3/D6 rotation cutoff). The exact error string
  //    "form_out_of_date" is a documented downstream signal - do not rename.
  if (!isAdmitted(fields.fingerprint, env)) {
    return submitResponse(env, 409, { error: "form_out_of_date" });
  }

  // 9. Per-IP rate limit (approximate, D6).
  const ip = request.headers.get("CF-Connecting-IP") ?? "unknown";
  const rate = await checkRateLimit(env, ip, config);
  if (!rate.allowed) {
    return submitResponse(env, 429, { error: "rate_limited" }, { "Retry-After": RETRY_AFTER_SECONDS });
  }

  // 10. Approximate total-blob cap (eventually-consistent list count, D6).
  if (await isOverCap(env, config)) {
    return submitResponse(env, 503, { error: "capacity_exceeded" }, { "Retry-After": RETRY_AFTER_SECONDS });
  }

  // 11-14. Generate receipt id, then the two fixed-order KV writes.
  return writeReceipt(env, config, rawBody, bytes.byteLength, fields.fingerprint);
}

interface EnvelopeFields {
  fingerprint: string;
}

/**
 * Parse the JSON body and extract ONLY the two cleartext routing fields the
 * relay validates (D6): recipient_key_fingerprint and ciphertext, each a
 * non-empty string. The fingerprint format is NOT validated here - the
 * admission allowlist is the real gate. The ciphertext is never decoded,
 * length-inspected beyond the overall body cap, or otherwise looked inside.
 * Returns null on any malformed/missing/wrong-type condition (all map to 400).
 */
function parseEnvelopeFields(rawBody: string): EnvelopeFields | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(rawBody);
  } catch {
    return null;
  }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    return null;
  }
  const obj = parsed as Record<string, unknown>;

  const fingerprint = obj["recipient_key_fingerprint"];
  if (typeof fingerprint !== "string" || fingerprint.length === 0) {
    return null;
  }
  const ciphertext = obj["ciphertext"];
  if (typeof ciphertext !== "string" || ciphertext.length === 0) {
    return null;
  }
  return { fingerprint };
}

/** Exact-string membership in the trimmed, comma-split admission allowlist. */
function isAdmitted(fingerprint: string, env: Env): boolean {
  const allow = env.ADMISSION_ALLOWLIST.split(",").map((entry) => entry.trim()).filter(Boolean);
  return allow.includes(fingerprint);
}

/**
 * Approximate blob-count cap: one plain KV list over the blob: prefix (D6 -
 * known-stale, eventually consistent by design; no caching or "smarter"
 * approximation). At pilot scale (~300 max) a single .list() page covers the
 * count; a partial listing beyond a page would only UNDER-count, which cannot
 * falsely trip the cap.
 */
async function isOverCap(env: Env, config: RelayConfig): Promise<boolean> {
  const listing = await env.INTAKE_BLOBS.list({ prefix: BLOB_PREFIX });
  return listing.keys.length >= config.approximateBlobCap;
}

/** 128 bits of randomness, lowercase hex, 32 chars, URL-safe by construction. */
function randomReceiptId(): string {
  return toHex(crypto.getRandomValues(new Uint8Array(16)));
}

/**
 * Steps 11-14: generate the receipt id and perform the two KV writes in a FIXED
 * ORDER (D6): the ciphertext blob completes BEFORE the ledger write is even
 * attempted; acknowledge (200) only after BOTH succeed. On blob-write failure,
 * no ledger write and a generic 500 (no orphan). On ledger-write failure after
 * a successful blob write, still 500 and NO rollback of the blob - the orphan
 * blob is the accepted, designed-for crash case, left for the puller to
 * discover via GET /receipts.
 */
async function writeReceipt(
  env: Env,
  config: RelayConfig,
  rawBody: string,
  size: number,
  fingerprint: string,
): Promise<Response> {
  const receiptId = randomReceiptId();

  try {
    // Store the ciphertext blob VERBATIM - the exact decoded string, no
    // JSON.parse/JSON.stringify round trip (byte-for-byte fidelity; the Rust
    // OuterEnvelope's flattened `extras` and key ordering must survive).
    await env.INTAKE_BLOBS.put(blobKey(receiptId), rawBody, {
      expirationTtl: config.blobTtlSeconds,
    });
  } catch (err) {
    console.error("relay_blob_write_failed", errorName(err));
    return submitResponse(env, 500, { error: "internal_error" });
  }

  const ledger = {
    receipt_id: receiptId,
    size,
    arrived_at: new Date().toISOString(),
    claimed_fingerprint: fingerprint, // content only, never ciphertext
  };
  try {
    await env.INTAKE_BLOBS.put(ledgerKey(receiptId), JSON.stringify(ledger), {
      expirationTtl: config.ledgerTtlSeconds,
    });
  } catch (err) {
    console.error("relay_ledger_write_failed", errorName(err));
    // Do NOT delete the just-written blob: the orphan is intentional (D6).
    return submitResponse(env, 500, { error: "internal_error" });
  }

  return submitResponse(env, 200, { receipt_id: receiptId });
}
