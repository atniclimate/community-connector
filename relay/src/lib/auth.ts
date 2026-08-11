import type { Env } from "../env.js";
import { fromHex, toHex } from "./hex.js";

/**
 * Control-plane bearer authentication (ADR-005 D6 "Control-plane credential").
 * The Worker holds only a SHA-256 HASH of the token (`env.CREDENTIAL_HASH`);
 * the token itself lives only on the pilot PC. On each control-plane request we
 * hash the presented token (Web Crypto `crypto.subtle.digest`, no external
 * crypto library per blueprint section 9) and compare against the stored hash
 * with a CONSTANT-TIME comparison.
 *
 * This is the single shared helper used identically by all three control-plane
 * routes; the check runs BEFORE any KV lookup, so a bad token never gets far
 * enough to distinguish "wrong token" from "right token, missing id" (D6: the
 * 401-vs-404 indistinguishability rule; auth failure returns a plain 404 at the
 * call site, never 401/403).
 */
export async function isAuthorized(request: Request, env: Env): Promise<boolean> {
  const header = request.headers.get("Authorization");
  if (header === null) {
    return false;
  }
  const scheme = "Bearer ";
  if (!header.startsWith(scheme)) {
    return false;
  }
  const token = header.slice(scheme.length);
  if (token.length === 0) {
    return false;
  }

  const presented = await sha256(token);
  const expected = fromHex(env.CREDENTIAL_HASH.trim());
  if (expected === null) {
    // Misconfigured/empty hash: a plain non-match, no distinct surface.
    return false;
  }
  return constantTimeEqual(presented, expected);
}

/** SHA-256 of a UTF-8 string, as raw digest bytes. */
async function sha256(input: string): Promise<Uint8Array> {
  const data = new TextEncoder().encode(input);
  const digest = await crypto.subtle.digest("SHA-256", data);
  return new Uint8Array(digest);
}

/** Lowercase hex SHA-256 of a UTF-8 string (exposed for tests/tooling). */
export async function sha256Hex(input: string): Promise<string> {
  return toHex(await sha256(input));
}

/**
 * Constant-time byte comparison: accumulate the OR of per-byte XOR differences
 * across EVERY byte and compare the accumulator to zero at the end - never
 * short-circuit on the first mismatch (D6 "constant-time compare"). The length
 * check is not a secret leak: both operands are fixed-size SHA-256 digests (32
 * bytes) and the presented value is a one-way hash of the token, so comparison
 * timing reveals nothing about the token.
 */
function constantTimeEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) {
    return false;
  }
  let diff = 0;
  for (let i = 0; i < a.length; i++) {
    diff |= (a[i] as number) ^ (b[i] as number);
  }
  return diff === 0;
}
