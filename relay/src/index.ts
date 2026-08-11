/**
 * Community Navigator intake relay - a zero-trust ciphertext store-and-forward
 * Worker. Implements ADR-005 D6 and docs/blueprints/intake-relay.md section 5.
 *
 * ADR-005 D1, verbatim: "The relay stores ciphertext only. It cannot read,
 * decrypt, validate, index, or transform payloads; its API is store-blob and
 * authenticated fetch/delete." There is no decryption key here and never will
 * be. Auto-purge of expired blobs is KV's own expirationTtl (set at write time),
 * not code here - no cron, no sweep loop.
 *
 * API contract (build exactly this):
 *   POST   /submit    public   accept one outer envelope
 *                              200 {receipt_id}; 400 malformed; 413 oversized;
 *                              409 {"error":"form_out_of_date"} not admitted;
 *                              429 rate limited; 503 approximate cap
 *   OPTIONS /submit   public   204 CORS preflight, no body
 *   GET    /receipts  bearer   200 {receipts,cursor}; 404 on auth failure
 *   GET    /blob/:id  bearer   200 stored envelope verbatim; 404 missing/auth
 *   DELETE /blob/:id  bearer   200 deleted now / 204 already gone; 404 auth
 *
 * Auth failure on any control-plane route is INDISTINGUISHABLE from
 * not-found: a plain 404, never 401/403 (D6). Any unrecognized path is a
 * generic 404; a known path with an unsupported method is a 405. Nothing thrown
 * anywhere escapes as Cloudflare's own error page - the top-level handler wraps
 * everything and returns our own generic 500.
 */
import type { Env } from "./env.js";
import { handleSubmit } from "./routes/submit.js";
import { handleReceipts } from "./routes/receipts.js";
import { handleGetBlob, handleDeleteBlob } from "./routes/blob.js";
import { submitPreflight } from "./lib/cors.js";
import { internalError, methodNotAllowed, notFound, errorName } from "./lib/http.js";

export default {
  async fetch(request: Request, env: Env, _ctx: ExecutionContext): Promise<Response> {
    try {
      return await route(request, env);
    } catch (err) {
      // Log the error CLASS only - never a message (it could echo user input) or
      // a stack trace. See ADR-005 D6 logging discipline.
      console.error("relay_unhandled_error", errorName(err));
      return internalError();
    }
  },
} satisfies ExportedHandler<Env>;

async function route(request: Request, env: Env): Promise<Response> {
  const { pathname } = new URL(request.url);
  const method = request.method;

  if (pathname === "/submit") {
    if (method === "POST") {
      return handleSubmit(request, env);
    }
    if (method === "OPTIONS") {
      return submitPreflight(env);
    }
    return methodNotAllowed("POST, OPTIONS");
  }

  if (pathname === "/receipts") {
    if (method === "GET") {
      return handleReceipts(request, env);
    }
    return methodNotAllowed("GET");
  }

  const receiptId = matchBlobId(pathname);
  if (receiptId !== null) {
    if (method === "GET") {
      return handleGetBlob(request, env, receiptId);
    }
    if (method === "DELETE") {
      return handleDeleteBlob(request, env, receiptId);
    }
    return methodNotAllowed("GET, DELETE");
  }

  // Unrecognized path: a generic 404, never a crash or a stack trace.
  return notFound();
}

/** Match `/blob/:id` and return a non-empty id, or null for any other shape. */
function matchBlobId(pathname: string): string | null {
  const segments = pathname.split("/").filter((s) => s.length > 0);
  if (segments.length === 2 && segments[0] === "blob") {
    return segments[1] ?? null;
  }
  return null;
}
