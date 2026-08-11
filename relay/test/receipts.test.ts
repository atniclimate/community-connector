import { env } from "cloudflare:test";
import { describe, expect, it } from "vitest";
import { bearer, dispatch, outerEnvelope, RELAY_ORIGIN, submitRequest } from "./helpers.js";

function receiptsRequest(headers: Record<string, string> = {}): Request {
  return new Request(`${RELAY_ORIGIN}/receipts`, { method: "GET", headers });
}

interface ReceiptRow {
  receipt_id: string;
  has_ledger: boolean;
  has_blob: boolean;
  size?: number;
  arrived_at?: string;
  claimed_fingerprint?: string;
}

describe("GET /receipts", () => {
  it("returns 404 (not 401) with no Authorization header", async () => {
    const res = await dispatch(receiptsRequest());
    expect(res.status).toBe(404);
    // Not an empty {"receipts":[]} - that would confirm the token was recognized.
    expect(await res.json()).toEqual({ error: "not_found" });
  });

  it("returns 404 for a wrong bearer token", async () => {
    const res = await dispatch(receiptsRequest(bearer("wrong-token")));
    expect(res.status).toBe(404);
  });

  it("returns 404 for a non-Bearer Authorization scheme", async () => {
    const res = await dispatch(receiptsRequest({ Authorization: "Basic abc123" }));
    expect(res.status).toBe(404);
  });

  it("carries NO CORS headers on the control plane", async () => {
    const res = await dispatch(receiptsRequest(bearer()));
    expect(res.status).toBe(200);
    expect(res.headers.get("Access-Control-Allow-Origin")).toBeNull();
  });

  it("returns an empty array (cursor null) when nothing has been submitted", async () => {
    const res = await dispatch(receiptsRequest(bearer()));
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ receipts: [], cursor: null });
  });

  it("lists two submissions with full metadata and both presence flags", async () => {
    const first = (await (await dispatch(submitRequest())).json()) as { receipt_id: string };
    const second = (await (await dispatch(submitRequest())).json()) as { receipt_id: string };

    const res = await dispatch(receiptsRequest(bearer()));
    const body = (await res.json()) as { receipts: ReceiptRow[]; cursor: null };
    expect(body.cursor).toBeNull();
    expect(body.receipts).toHaveLength(2);

    const byId = new Map(body.receipts.map((r) => [r.receipt_id, r]));
    for (const id of [first.receipt_id, second.receipt_id]) {
      const row = byId.get(id);
      expect(row).toBeDefined();
      expect(row?.has_blob).toBe(true);
      expect(row?.has_ledger).toBe(true);
      expect(typeof row?.size).toBe("number");
      expect(typeof row?.arrived_at).toBe("string");
      expect(row?.claimed_fingerprint).toBe(outerEnvelope()["recipient_key_fingerprint"]);
    }
  });

  it("surfaces an orphan blob (blob present, no ledger) via the union of both prefixes", async () => {
    // Simulate the ledger-write-failure crash case: a blob with no ledger entry.
    await env.INTAKE_BLOBS.put("blob:orphan-xyz", JSON.stringify(outerEnvelope()));

    const res = await dispatch(receiptsRequest(bearer()));
    const body = (await res.json()) as { receipts: ReceiptRow[] };
    const orphan = body.receipts.find((r) => r.receipt_id === "orphan-xyz");
    expect(orphan).toBeDefined();
    expect(orphan?.has_blob).toBe(true);
    expect(orphan?.has_ledger).toBe(false);
    // No ledger metadata is surfaced for an orphan.
    expect(orphan?.size).toBeUndefined();
    expect(orphan?.arrived_at).toBeUndefined();
    expect(orphan?.claimed_fingerprint).toBeUndefined();
  });

  it("surfaces a ledger-without-blob receipt (already deleted or expired)", async () => {
    await env.INTAKE_BLOBS.put(
      "ledger:gone-1",
      JSON.stringify({ version: 1, receipt_id: "gone-1", size: 42, arrived_at: "2026-08-11T00:00:00.000Z", claimed_fingerprint: "aaaa-bbbb-cccc-dddd" }),
    );
    const res = await dispatch(receiptsRequest(bearer()));
    const body = (await res.json()) as { receipts: ReceiptRow[] };
    const row = body.receipts.find((r) => r.receipt_id === "gone-1");
    expect(row?.has_ledger).toBe(true);
    expect(row?.has_blob).toBe(false);
    expect(row?.size).toBe(42);
  });

  it("downgrades a ledger row whose schema version does not match (drift detected, metadata withheld)", async () => {
    // I7: a row written under a different ledger schema version must be a
    // detectable, downgraded row - presence flag kept, metadata NOT surfaced -
    // never a shape-guessed read of a stale/foreign schema.
    await env.INTAKE_BLOBS.put(
      "ledger:drifted-1",
      JSON.stringify({ version: 999, receipt_id: "drifted-1", size: 7, arrived_at: "2026-08-11T00:00:00.000Z", claimed_fingerprint: "aaaa-bbbb-cccc-dddd" }),
    );
    const res = await dispatch(receiptsRequest(bearer()));
    const body = (await res.json()) as { receipts: ReceiptRow[] };
    const row = body.receipts.find((r) => r.receipt_id === "drifted-1");
    expect(row?.has_ledger).toBe(true);
    expect(row?.size).toBeUndefined();
    expect(row?.arrived_at).toBeUndefined();
    expect(row?.claimed_fingerprint).toBeUndefined();
  });
});
