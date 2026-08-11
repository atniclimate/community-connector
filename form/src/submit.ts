/**
 * Networking to the relay's POST /submit endpoint (ADR-005 D6, blueprint 5.1).
 *
 * The status-code -> outcome mapping is a PURE function (classifyResponse),
 * unit-testable without a real network stack, kept separate from the
 * fetch-calling code (postEnvelope) - mirroring the model/render split.
 *
 * RETRY CONTRACT (blueprint 4.1): sealing uses a fresh ephemeral key per call
 * (crypto.rs `seal_is_anonymous_and_nondeterministic`), so a client that
 * reseals on every retry click could make the relay store two blobs for one
 * attempt. The puller's semantic dedup collapses that to a no-op ONLY because
 * `submission_id` stays the same. Therefore: build + seal the OuterEnvelope
 * exactly ONCE per submit attempt; on a retryable failure, re-POST that SAME
 * object (postEnvelope does not reseal). Only a genuinely new submission (after
 * a confirmed receipt / start-over) gets a fresh submission_id and a fresh seal.
 */
import type { OuterEnvelope } from "./envelope";

export type SubmitOutcome =
  | { readonly kind: "accepted"; readonly receiptId: string }
  /** 409: the embedded key's fingerprint is off the relay's admission allowlist
   * (rotated out). Retrying the same stale envelope can never succeed. */
  | { readonly kind: "form_out_of_date" }
  /** 429: rate-limited; Retry-After may be present. Do not hammer retries. */
  | { readonly kind: "rate_limited"; readonly retryAfter: string | null }
  /** 503: relay over capacity; retryable. */
  | { readonly kind: "unavailable" }
  /** Any other non-2xx, or a 2xx without a usable receipt_id (protocol error). */
  | { readonly kind: "error"; readonly status: number }
  /** fetch threw - relay unreachable/offline; the request did not complete. */
  | { readonly kind: "network_error" };

const HTTP_CONFLICT = 409;
const HTTP_TOO_MANY_REQUESTS = 429;
const HTTP_SERVICE_UNAVAILABLE = 503;

function receiptIdOf(body: unknown): string | null {
  if (typeof body === "object" && body !== null) {
    const value = (body as { readonly receipt_id?: unknown }).receipt_id;
    if (typeof value === "string" && value.length > 0) {
      return value;
    }
  }
  return null;
}

/**
 * Pure mapping from a completed HTTP response to a SubmitOutcome. `status` is
 * the response status; `body` is the parsed JSON body (or null/undefined if it
 * did not parse); `retryAfter` is the Retry-After header value (or null).
 *
 * A 2xx is success ONLY if the body carries a non-empty string `receipt_id`
 * (ADR-005 D6: the receipt id is the acknowledgement). A 2xx without one is a
 * protocol error, not a silent success. The specific 2xx code is not assumed.
 */
export function classifyResponse(
  status: number,
  body: unknown,
  retryAfter: string | null,
): SubmitOutcome {
  if (status >= 200 && status < 300) {
    const receiptId = receiptIdOf(body);
    return receiptId !== null
      ? { kind: "accepted", receiptId }
      : { kind: "error", status };
  }
  switch (status) {
    case HTTP_CONFLICT:
      return { kind: "form_out_of_date" };
    case HTTP_TOO_MANY_REQUESTS:
      return { kind: "rate_limited", retryAfter };
    case HTTP_SERVICE_UNAVAILABLE:
      return { kind: "unavailable" };
    default:
      return { kind: "error", status };
  }
}

/**
 * POSTs an already-sealed OuterEnvelope to `relayOrigin + "/submit"` and
 * classifies the result. Never reseals - pass the SAME `outer` object on retry
 * (see the retry contract above). A thrown fetch (no response) maps to
 * network_error; a completed response is classified by classifyResponse.
 *
 * Note (CORS): a cross-origin JSON POST triggers a browser preflight (OPTIONS)
 * that the relay (step 5) handles; a mocked fetch in a unit test does not
 * exercise real CORS, so a passing unit test is not proof cross-origin POST
 * works against a deployed Worker - that is step 9's end-to-end rehearsal.
 */
export async function postEnvelope(
  relayOrigin: string,
  outer: OuterEnvelope,
  fetchImpl: typeof fetch = fetch,
): Promise<SubmitOutcome> {
  let response: Response;
  try {
    response = await fetchImpl(`${relayOrigin}/submit`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(outer),
    });
  } catch {
    return { kind: "network_error" };
  }
  let body: unknown = null;
  try {
    body = await response.json();
  } catch {
    body = null;
  }
  return classifyResponse(response.status, body, response.headers.get("Retry-After"));
}
