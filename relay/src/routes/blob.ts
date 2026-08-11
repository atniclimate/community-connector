import type { Env } from "../env.js";
import { isAuthorized } from "../lib/auth.js";
import { blobKey, ledgerKey } from "../lib/kv.js";
import { jsonResponse, notFound } from "../lib/http.js";

/**
 * GET /blob/:id (authenticated). Returns the stored outer envelope byte-for-byte
 * unmodified. Reads are NON-DESTRUCTIVE and repeatable (D6 withdraws the earlier
 * "fetch-then-delete" design): two consecutive GETs on the same id behave
 * identically. Auth failure and missing-id are both a plain 404 (D6: no
 * existence oracle; a receipt id grants no lookup capability) - the auth check
 * runs before any KV lookup, so timing cannot separate the two.
 */
export async function handleGetBlob(request: Request, env: Env, receiptId: string): Promise<Response> {
  if (!(await isAuthorized(request, env))) {
    return notFound();
  }
  // Read the RAW stored bytes (submit stores the verbatim request bytes as an
  // ArrayBuffer). Returning bytes, not a re-decoded string, keeps the response
  // body byte-for-byte identical to the originally-POSTed body even when it was
  // valid JSON but not clean UTF-8. No CORS (control plane).
  const value = await env.INTAKE_BLOBS.get(blobKey(receiptId), "arrayBuffer");
  if (value === null) {
    return notFound();
  }
  return new Response(value, {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

/**
 * DELETE /blob/:id (authenticated, idempotent). Deletes BOTH blob:<id> and
 * ledger:<id> (blueprint 5.1: "Deletes blob + ledger entry") - an explicit
 * delete does not leave the ledger behind (the ledger only outlives the blob via
 * its longer TTL when nothing explicitly deleted it). Idempotency tracks the
 * BLOB's existence before deletion (the route is named for the blob): existed ->
 * delete both, 200; did not exist -> 204 (best-effort delete the ledger too).
 * Both are success; the distinct codes let a caller tell "I just deleted
 * something" from "there was already nothing there" (D6 idempotent delete).
 * Auth failure -> plain 404.
 */
export async function handleDeleteBlob(request: Request, env: Env, receiptId: string): Promise<Response> {
  if (!(await isAuthorized(request, env))) {
    return notFound();
  }
  const existed = (await env.INTAKE_BLOBS.get(blobKey(receiptId))) !== null;

  // Delete both halves regardless of ledger state; the response code tracks the
  // blob's prior presence.
  await env.INTAKE_BLOBS.delete(blobKey(receiptId));
  await env.INTAKE_BLOBS.delete(ledgerKey(receiptId));

  if (existed) {
    return jsonResponse(200, { deleted: true });
  }
  return new Response(null, { status: 204 });
}
