import { env } from "cloudflare:test";
import { describe, expect, it } from "vitest";
import { bearer, dispatch, RELAY_ORIGIN, submitRequest } from "./helpers.js";

function blobRequest(id: string, method: string, headers: Record<string, string> = {}): Request {
  return new Request(`${RELAY_ORIGIN}/blob/${id}`, { method, headers });
}

async function submitAndGetId(body?: string): Promise<string> {
  const res = await dispatch(submitRequest(body === undefined ? {} : { body }));
  const json = (await res.json()) as { receipt_id: string };
  return json.receipt_id;
}

describe("GET /blob/:id", () => {
  it("returns 404 with no auth", async () => {
    const res = await dispatch(blobRequest("anything", "GET"));
    expect(res.status).toBe(404);
  });

  it("returns 404 for a wrong token", async () => {
    const res = await dispatch(blobRequest("anything", "GET", bearer("nope")));
    expect(res.status).toBe(404);
  });

  it("returns 404 for an authenticated request to a missing id", async () => {
    const res = await dispatch(blobRequest("does-not-exist", "GET", bearer()));
    expect(res.status).toBe(404);
  });

  it("returns 200 with the byte-identical stored envelope and JSON content type", async () => {
    // An odd-ordered body with a multibyte char catches any accidental re-serialization.
    const body =
      '{"ciphertext":"Yg==","z_extra":"kéep","recipient_key_fingerprint":"3f9a-1c02-7b41-e6d5","intake_envelope_version":"0.1.0"}';
    const id = await submitAndGetId(body);

    const res = await dispatch(blobRequest(id, "GET", bearer()));
    expect(res.status).toBe(200);
    expect(res.headers.get("Content-Type")).toContain("application/json");
    expect(await res.text()).toBe(body);
    // No CORS on the control plane.
    expect(res.headers.get("Access-Control-Allow-Origin")).toBeNull();
  });

  it("is non-destructive: two consecutive GETs behave identically", async () => {
    const id = await submitAndGetId();
    const first = await dispatch(blobRequest(id, "GET", bearer()));
    const second = await dispatch(blobRequest(id, "GET", bearer()));
    expect(first.status).toBe(200);
    expect(second.status).toBe(200);
    expect(await first.text()).toBe(await second.text());
  });
});

describe("DELETE /blob/:id", () => {
  it("returns 404 with no auth", async () => {
    const res = await dispatch(blobRequest("anything", "DELETE"));
    expect(res.status).toBe(404);
  });

  it("deletes both the blob and the ledger entry and returns 200", async () => {
    const id = await submitAndGetId();
    expect(await env.INTAKE_BLOBS.get(`blob:${id}`)).not.toBeNull();
    expect(await env.INTAKE_BLOBS.get(`ledger:${id}`)).not.toBeNull();

    const res = await dispatch(blobRequest(id, "DELETE", bearer()));
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ deleted: true });

    expect(await env.INTAKE_BLOBS.get(`blob:${id}`)).toBeNull();
    expect(await env.INTAKE_BLOBS.get(`ledger:${id}`)).toBeNull();
  });

  it("returns 204 (not 200) for an id that never existed", async () => {
    const res = await dispatch(blobRequest("never-existed", "DELETE", bearer()));
    expect(res.status).toBe(204);
    expect(await res.text()).toBe("");
  });

  it("is idempotent end-to-end: first delete 200, second delete 204", async () => {
    const id = await submitAndGetId();
    const first = await dispatch(blobRequest(id, "DELETE", bearer()));
    const second = await dispatch(blobRequest(id, "DELETE", bearer()));
    expect(first.status).toBe(200);
    expect(second.status).toBe(204);
  });

  it("removes the receipt from a subsequent GET /receipts listing", async () => {
    const id = await submitAndGetId();
    await dispatch(blobRequest(id, "DELETE", bearer()));
    const listing = await dispatch(new Request(`${RELAY_ORIGIN}/receipts`, { method: "GET", headers: bearer() }));
    const body = (await listing.json()) as { receipts: Array<{ receipt_id: string }> };
    expect(body.receipts.some((r) => r.receipt_id === id)).toBe(false);
  });
});
