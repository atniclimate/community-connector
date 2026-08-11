import { env, createExecutionContext, waitOnExecutionContext } from "cloudflare:test";
import worker from "../src/index.js";
import type { Env } from "../src/env.js";

/**
 * The control-plane bearer token whose SHA-256 hex is seeded as CREDENTIAL_HASH
 * in vitest.config.ts. Tests send `Authorization: Bearer ${TEST_BEARER}`; the
 * Worker hashes the presented token and constant-time-compares it.
 */
export const TEST_BEARER = "cn-relay-test-bearer";

/** A fingerprint present on the seeded ADMISSION_ALLOWLIST. */
export const ADMITTED_FINGERPRINT = "3f9a-1c02-7b41-e6d5";
/** A syntactically-fine fingerprint that is NOT on the allowlist. */
export const UNADMITTED_FINGERPRINT = "dead-beef-0000-1111";

export const RELAY_ORIGIN = "https://relay.test";

/** A minimal well-formed outer envelope (relay treats ciphertext as opaque). */
export function outerEnvelope(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    intake_envelope_version: "0.1.0",
    recipient_key_fingerprint: ADMITTED_FINGERPRINT,
    ciphertext: "c2VhbGVkLWJveC1jaXBoZXJ0ZXh0", // opaque base64-looking string
    ...overrides,
  };
}

export interface SubmitInit {
  body?: string;
  contentType?: string | null;
  contentLength?: string | null;
  ip?: string;
}

/** Build a POST /submit request. */
export function submitRequest(init: SubmitInit = {}): Request {
  const headers = new Headers();
  if (init.contentType !== null) {
    headers.set("Content-Type", init.contentType ?? "application/json");
  }
  if (init.contentLength != null) {
    headers.set("Content-Length", init.contentLength);
  }
  headers.set("CF-Connecting-IP", init.ip ?? "203.0.113.7");
  const body = init.body ?? JSON.stringify(outerEnvelope());
  return new Request(`${RELAY_ORIGIN}/submit`, { method: "POST", headers, body });
}

/** Authorization header for a given bearer token (defaults to the valid one). */
export function bearer(token: string = TEST_BEARER): Record<string, string> {
  return { Authorization: `Bearer ${token}` };
}

/** Dispatch a request through the real Worker, optionally with an env override. */
export async function dispatch(request: Request, overrideEnv?: Env): Promise<Response> {
  const ctx = createExecutionContext();
  const res = await worker.fetch(request, overrideEnv ?? env, ctx);
  await waitOnExecutionContext(ctx);
  return res;
}

/** The base test env (typed), for building overrides via spread. */
export function baseEnv(): Env {
  return env;
}

/**
 * Wrap a KVNamespace so every .put records its key in order. Used to prove the
 * blob write is attempted before the ledger write (D6 fixed write order).
 */
export function recordingKv(kv: KVNamespace, putLog: string[]): KVNamespace {
  return new Proxy(kv, {
    get(target, prop, receiver) {
      if (prop === "put") {
        return (key: string, value: unknown, options?: unknown) => {
          putLog.push(key);
          return (target.put as (k: string, v: unknown, o?: unknown) => Promise<void>)(key, value, options);
        };
      }
      const value = Reflect.get(target, prop, receiver);
      return typeof value === "function" ? value.bind(target) : value;
    },
  }) as KVNamespace;
}

/**
 * Wrap a KVNamespace so .put throws for keys matching `predicate` (and succeeds
 * otherwise). Used to force a blob- or ledger-write failure and prove the
 * generic 500 path with no rollback.
 */
export function throwingKv(kv: KVNamespace, predicate: (key: string) => boolean): KVNamespace {
  return new Proxy(kv, {
    get(target, prop, receiver) {
      if (prop === "put") {
        return (key: string, value: unknown, options?: unknown) => {
          if (predicate(key)) {
            throw new Error("kv put boom");
          }
          return (target.put as (k: string, v: unknown, o?: unknown) => Promise<void>)(key, value, options);
        };
      }
      const value = Reflect.get(target, prop, receiver);
      return typeof value === "function" ? value.bind(target) : value;
    },
  }) as KVNamespace;
}
