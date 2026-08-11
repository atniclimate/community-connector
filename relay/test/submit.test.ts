import { env } from "cloudflare:test";
import { describe, expect, it } from "vitest";
import type { Env } from "../src/env.js";
import {
  ADMITTED_FINGERPRINT,
  UNADMITTED_FINGERPRINT,
  dispatch,
  outerEnvelope,
  recordingKv,
  RELAY_ORIGIN,
  submitRequest,
  throwingKv,
} from "./helpers.js";

function withEnv(overrides: Partial<Record<keyof Env, unknown>>): Env {
  return { ...env, ...overrides } as Env;
}

describe("POST /submit", () => {
  it("accepts a valid, admitted envelope: 200, receipt id, both KV keys written", async () => {
    const body = JSON.stringify(outerEnvelope());
    const res = await dispatch(submitRequest({ body }));

    expect(res.status).toBe(200);
    const json = (await res.json()) as { receipt_id: string };
    expect(json.receipt_id).toMatch(/^[0-9a-f]{32}$/);

    const blob = await env.INTAKE_BLOBS.get(`blob:${json.receipt_id}`);
    expect(blob).toBe(body); // stored verbatim

    const ledgerRaw = await env.INTAKE_BLOBS.get(`ledger:${json.receipt_id}`);
    expect(ledgerRaw).not.toBeNull();
    const ledger = JSON.parse(ledgerRaw as string) as Record<string, unknown>;
    expect(ledger["receipt_id"]).toBe(json.receipt_id);
    expect(ledger["size"]).toBe(new TextEncoder().encode(body).byteLength);
    expect(ledger["claimed_fingerprint"]).toBe(ADMITTED_FINGERPRINT);
    expect(typeof ledger["arrived_at"]).toBe("string");
  });

  it("stores the body byte-for-byte, no JSON re-serialization", async () => {
    // Deliberately odd key order + an extra field + a multibyte char.
    const body =
      '{"ciphertext":"Yg==","z_extra":"kéep","recipient_key_fingerprint":"3f9a-1c02-7b41-e6d5","intake_envelope_version":"0.1.0"}';
    const res = await dispatch(submitRequest({ body }));
    expect(res.status).toBe(200);
    const { receipt_id } = (await res.json()) as { receipt_id: string };
    expect(await env.INTAKE_BLOBS.get(`blob:${receipt_id}`)).toBe(body);
  });

  it("rejects malformed JSON with 400", async () => {
    const res = await dispatch(submitRequest({ body: "{not json" }));
    expect(res.status).toBe(400);
  });

  it("rejects a JSON array (non-object) with 400", async () => {
    const res = await dispatch(submitRequest({ body: "[]" }));
    expect(res.status).toBe(400);
  });

  it("rejects a missing recipient_key_fingerprint with 400", async () => {
    const body = JSON.stringify({ intake_envelope_version: "0.1.0", ciphertext: "abc" });
    const res = await dispatch(submitRequest({ body }));
    expect(res.status).toBe(400);
  });

  it("rejects a missing ciphertext with 400", async () => {
    const body = JSON.stringify({
      intake_envelope_version: "0.1.0",
      recipient_key_fingerprint: ADMITTED_FINGERPRINT,
    });
    const res = await dispatch(submitRequest({ body }));
    expect(res.status).toBe(400);
  });

  it("rejects a wrong Content-Type with 400", async () => {
    const res = await dispatch(submitRequest({ contentType: "text/plain" }));
    expect(res.status).toBe(400);
  });

  it("does NOT check intake_envelope_version (stores an unknown-major envelope)", async () => {
    // Version checking is the puller's job (D3), not the relay's.
    const body = JSON.stringify(outerEnvelope({ intake_envelope_version: "9.9.9" }));
    const res = await dispatch(submitRequest({ body }));
    expect(res.status).toBe(200);
  });

  it("rejects an oversized body when Content-Length under-claims the size (413)", async () => {
    const small = withEnv({ MAX_BLOB_SIZE_BYTES: "50" });
    const body = JSON.stringify(outerEnvelope({ ciphertext: "x".repeat(200) }));
    // Lie: declare far under the cap; the real byte-length check must still fire.
    const res = await dispatch(submitRequest({ body, contentLength: "10" }), small);
    expect(res.status).toBe(413);
  });

  it("rejects an oversized body with no Content-Length header at all (413)", async () => {
    const small = withEnv({ MAX_BLOB_SIZE_BYTES: "50" });
    const body = JSON.stringify(outerEnvelope({ ciphertext: "x".repeat(200) }));
    const headers = new Headers({ "Content-Type": "application/json", "CF-Connecting-IP": "203.0.113.9" });
    const req = new Request(`${RELAY_ORIGIN}/submit`, { method: "POST", headers, body });
    // Precondition: a string body here carries no Content-Length header, so the
    // byte-length check (not the header pre-check) is what rejects it.
    expect(req.headers.get("Content-Length")).toBeNull();
    const res = await dispatch(req, small);
    expect(res.status).toBe(413);
  });

  it("rejects a fingerprint not on the allowlist with 409 form_out_of_date", async () => {
    const body = JSON.stringify(outerEnvelope({ recipient_key_fingerprint: UNADMITTED_FINGERPRINT }));
    const res = await dispatch(submitRequest({ body }));
    expect(res.status).toBe(409);
    expect(await res.json()).toEqual({ error: "form_out_of_date" });
  });

  it("rate-limits one IP after the limit and leaves a different IP unaffected", async () => {
    const limited = withEnv({ RATE_LIMIT_PER_IP_PER_MINUTE: "3" });
    const ip = "198.51.100.4";
    const statuses: number[] = [];
    for (let i = 0; i < 4; i++) {
      const res = await dispatch(submitRequest({ ip }), limited);
      statuses.push(res.status);
    }
    expect(statuses.slice(0, 3)).toEqual([200, 200, 200]);
    const blocked = statuses[3];
    expect(blocked).toBe(429);

    // The 4th (blocked) response carries Retry-After.
    const again = await dispatch(submitRequest({ ip }), limited);
    expect(again.status).toBe(429);
    expect(again.headers.get("Retry-After")).not.toBeNull();

    // A different IP in the same window is unaffected (per-IP scoping).
    const other = await dispatch(submitRequest({ ip: "198.51.100.5" }), limited);
    expect(other.status).toBe(200);
  });

  it("refuses with 503 + Retry-After when the approximate blob cap is reached", async () => {
    const capped = withEnv({ APPROXIMATE_BLOB_CAP: "2" });
    await env.INTAKE_BLOBS.put("blob:seed-a", "{}");
    await env.INTAKE_BLOBS.put("blob:seed-b", "{}");
    const res = await dispatch(submitRequest(), capped);
    expect(res.status).toBe(503);
    expect(res.headers.get("Retry-After")).not.toBeNull();
  });

  it("writes the blob BEFORE the ledger (fixed write order)", async () => {
    const putLog: string[] = [];
    const spied = withEnv({ INTAKE_BLOBS: recordingKv(env.INTAKE_BLOBS, putLog) });
    const res = await dispatch(submitRequest(), spied);
    expect(res.status).toBe(200);
    const blobIdx = putLog.findIndex((k) => k.startsWith("blob:"));
    const ledgerIdx = putLog.findIndex((k) => k.startsWith("ledger:"));
    expect(blobIdx).toBeGreaterThanOrEqual(0);
    expect(ledgerIdx).toBeGreaterThanOrEqual(0);
    expect(blobIdx).toBeLessThan(ledgerIdx);
  });

  it("returns 500 and writes NO ledger when the blob write fails", async () => {
    const putLog: string[] = [];
    const failing = withEnv({
      INTAKE_BLOBS: throwingKv(recordingKv(env.INTAKE_BLOBS, putLog), (k) => k.startsWith("blob:")),
    });
    const res = await dispatch(submitRequest(), failing);
    expect(res.status).toBe(500);
    expect(await res.json()).toEqual({ error: "internal_error" });
    // No ledger put was attempted after the blob write threw.
    expect(putLog.some((k) => k.startsWith("ledger:"))).toBe(false);
  });

  it("returns 500 and does NOT roll back the blob when the ledger write fails", async () => {
    const failing = withEnv({
      INTAKE_BLOBS: throwingKv(env.INTAKE_BLOBS, (k) => k.startsWith("ledger:")),
    });
    const res = await dispatch(submitRequest(), failing);
    expect(res.status).toBe(500);
    // The orphan blob is intentionally left behind (found later via GET /receipts).
    const blobs = await env.INTAKE_BLOBS.list({ prefix: "blob:" });
    expect(blobs.keys.length).toBe(1);
  });

  it("produces a different, correctly-shaped receipt id on each call", async () => {
    const a = (await (await dispatch(submitRequest())).json()) as { receipt_id: string };
    const b = (await (await dispatch(submitRequest())).json()) as { receipt_id: string };
    expect(a.receipt_id).toMatch(/^[0-9a-f]{32}$/);
    expect(b.receipt_id).toMatch(/^[0-9a-f]{32}$/);
    expect(a.receipt_id).not.toBe(b.receipt_id);
  });

  it("carries the CORS origin header on a successful POST response", async () => {
    const res = await dispatch(submitRequest());
    expect(res.status).toBe(200);
    expect(res.headers.get("Access-Control-Allow-Origin")).toBe(env.PAGES_ORIGIN);
  });

  it("carries the CORS origin header on an error POST response too", async () => {
    const body = JSON.stringify(outerEnvelope({ recipient_key_fingerprint: UNADMITTED_FINGERPRINT }));
    const res = await dispatch(submitRequest({ body }));
    expect(res.status).toBe(409);
    expect(res.headers.get("Access-Control-Allow-Origin")).toBe(env.PAGES_ORIGIN);
  });
});

describe("OPTIONS /submit (CORS preflight)", () => {
  it("returns 204 with CORS headers and no body", async () => {
    const req = new Request(`${RELAY_ORIGIN}/submit`, { method: "OPTIONS" });
    const res = await dispatch(req);
    expect(res.status).toBe(204);
    expect(res.headers.get("Access-Control-Allow-Origin")).toBe(env.PAGES_ORIGIN);
    expect(res.headers.get("Access-Control-Allow-Methods")).toContain("POST");
    expect(res.headers.get("Access-Control-Allow-Headers")).toContain("Content-Type");
    expect(await res.text()).toBe("");
  });
});
