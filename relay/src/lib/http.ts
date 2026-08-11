/**
 * Typed response helpers for the CONTROL PLANE and generic failures. Every
 * failure path returns a small, generic, machine-readable JSON body and never a
 * stack trace, exception message, internal path, or KV key name (ADR-005 D6
 * "never leak internal state").
 *
 * Control-plane routes (GET /receipts, GET|DELETE /blob/:id) carry NO CORS
 * headers at all (D5: "No CORS for the control plane"). CORS for /submit is a
 * separate concern handled in cors.ts.
 */

const JSON_HEADERS: Record<string, string> = {
  "Content-Type": "application/json",
};

/** A JSON response with a status code. Used by the control plane; no CORS. */
export function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { ...JSON_HEADERS } });
}

/**
 * The single generic 404. Auth failure on any control-plane route returns
 * EXACTLY this, identical to a genuine missing-resource 404 (D6 401-vs-404
 * rule: "401 indistinguishable from 404 for unauthenticated"; no existence
 * oracle). Callers must never return an empty typed body (e.g. {"receipts":[]})
 * on auth failure - that would itself confirm the token was recognized.
 */
export function notFound(): Response {
  return jsonResponse(404, { error: "not_found" });
}

/** 405 for a known path reached with an unsupported method. */
export function methodNotAllowed(allow: string): Response {
  return new Response(JSON.stringify({ error: "method_not_allowed" }), {
    status: 405,
    headers: { ...JSON_HEADERS, Allow: allow },
  });
}

/**
 * The generic 500 for any unexpected failure (a KV op throwing, a config parse
 * error, anything not classified above). Body carries no internal detail.
 */
export function internalError(): Response {
  return jsonResponse(500, { error: "internal_error" });
}

/** Error class/type name for logging - never the message (which could echo input). */
export function errorName(err: unknown): string {
  return err instanceof Error ? err.name : "unknown";
}
