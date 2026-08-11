import type { Env } from "../env.js";

/**
 * CORS for POST /submit ONLY (ADR-005 D5, blueprint 5.2). This is a hard split:
 * /submit and its OPTIONS preflight carry these headers; the control plane
 * (GET /receipts, GET|DELETE /blob/:id) carries NONE - it is called by the
 * native puller, not a browser, so any Access-Control-* header there would be
 * wrong even if narrowly scoped.
 *
 * `Access-Control-Allow-Origin` is the literal configured Pages origin, never
 * "*". Every /submit response (success AND every error path) must carry it, or
 * the browser blocks the form from reading a result the request already
 * produced server-side - hence submitResponse/submitPreflight below are the
 * only construction sites for /submit responses.
 */

/** The 204 CORS preflight for OPTIONS /submit: CORS headers, no body. */
export function submitPreflight(env: Env): Response {
  return new Response(null, {
    status: 204,
    headers: {
      "Access-Control-Allow-Origin": env.PAGES_ORIGIN,
      "Access-Control-Allow-Methods": "POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
      "Access-Control-Max-Age": "3600",
    },
  });
}

/**
 * Build a POST /submit response with the required CORS origin header always
 * attached. `body === null` yields an empty body (used only on success paths
 * that carry a body, and never here - kept for symmetry); a JSON body sets the
 * JSON content type. `extraHeaders` carries e.g. Retry-After on 429/503.
 */
export function submitResponse(
  env: Env,
  status: number,
  body: unknown,
  extraHeaders?: Record<string, string>,
): Response {
  const headers = new Headers({
    "Access-Control-Allow-Origin": env.PAGES_ORIGIN,
    "Content-Type": "application/json",
    ...extraHeaders,
  });
  return new Response(JSON.stringify(body), { status, headers });
}
