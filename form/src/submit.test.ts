import { beforeAll, describe, expect, it, vi } from "vitest";

import { ready, hexToBytes } from "./crypto";
import { buildInnerPayload, buildOuterEnvelope, type OuterEnvelope } from "./envelope";
import { classifyResponse, postEnvelope } from "./submit";

const TEST_PUBLIC_KEY_HEX = "205fd0dce7b409a3231b86dad10f6e3a276bb2e0838bc605501131f96b117a71";
const TEST_FINGERPRINT = "efdf-7ce7-69fa-feeb-7512-a100-6450-45c6";

describe("classifyResponse (pure)", () => {
  it("accepts a 2xx carrying a receipt_id", () => {
    expect(classifyResponse(200, { receipt_id: "r-1" }, null)).toEqual({
      kind: "accepted",
      receiptId: "r-1",
    });
    expect(classifyResponse(201, { receipt_id: "r-2" }, null)).toEqual({
      kind: "accepted",
      receiptId: "r-2",
    });
  });

  it("treats a 2xx WITHOUT a usable receipt_id as a protocol error", () => {
    expect(classifyResponse(200, {}, null)).toEqual({ kind: "error", status: 200 });
    expect(classifyResponse(200, { receipt_id: "" }, null)).toEqual({ kind: "error", status: 200 });
    expect(classifyResponse(200, null, null)).toEqual({ kind: "error", status: 200 });
  });

  it("maps 409 -> form_out_of_date", () => {
    expect(classifyResponse(409, { error: "form_out_of_date" }, null)).toEqual({
      kind: "form_out_of_date",
    });
  });

  it("maps 429 -> rate_limited with Retry-After", () => {
    expect(classifyResponse(429, null, "30")).toEqual({ kind: "rate_limited", retryAfter: "30" });
    expect(classifyResponse(429, null, null)).toEqual({ kind: "rate_limited", retryAfter: null });
  });

  it("maps 503 -> unavailable", () => {
    expect(classifyResponse(503, null, null)).toEqual({ kind: "unavailable" });
  });

  it("maps other non-2xx -> error(status)", () => {
    expect(classifyResponse(500, null, null)).toEqual({ kind: "error", status: 500 });
    expect(classifyResponse(400, null, null)).toEqual({ kind: "error", status: 400 });
  });
});

describe("postEnvelope network handling", () => {
  it("maps a thrown fetch to network_error", async () => {
    const outer: OuterEnvelope = {
      intake_envelope_version: "0.1.0",
      recipient_key_fingerprint: TEST_FINGERPRINT,
      ciphertext: "AAAA",
    };
    const fetchImpl = vi.fn(async () => {
      throw new Error("offline");
    });
    const outcome = await postEnvelope("http://localhost:8787", outer, fetchImpl as unknown as typeof fetch);
    expect(outcome).toEqual({ kind: "network_error" });
  });
});

describe("retry reuses the exact sealed bytes (no reseal)", () => {
  beforeAll(async () => {
    await ready();
  });

  it("POSTs a byte-identical body on a network-error-then-retry", async () => {
    // Seal ONCE; the same OuterEnvelope object is re-POSTed on retry.
    const inner = buildInnerPayload({
      submissionId: "00000000-0000-4000-8000-000000000000",
      formVersion: "remote-draft-2026-08-11",
      kind: "person",
      fields: { display_name: "Synthetic Person" },
      consentTextDigest: "ae5f7cc0d740726e81785919d1dbe6ad715895c94da278610da354d730bcd36c",
      consentAffirmed: true,
      capturedAt: "2026-08-11T00:00:00.000Z",
      consentAffirmedAt: "2026-08-11T00:00:00.000Z",
    });
    const outer = buildOuterEnvelope(inner, hexToBytes(TEST_PUBLIC_KEY_HEX), TEST_FINGERPRINT);

    const bodies: string[] = [];
    let call = 0;
    const fetchImpl = vi.fn(async (_url: string, init?: RequestInit) => {
      bodies.push(String(init?.body));
      call += 1;
      if (call === 1) {
        throw new Error("network down"); // first attempt fails
      }
      return new Response(JSON.stringify({ receipt_id: "r-9" }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    });

    const first = await postEnvelope("http://localhost:8787", outer, fetchImpl as unknown as typeof fetch);
    expect(first).toEqual({ kind: "network_error" });
    const second = await postEnvelope("http://localhost:8787", outer, fetchImpl as unknown as typeof fetch);
    expect(second).toEqual({ kind: "accepted", receiptId: "r-9" });

    // Byte-identical body on both attempts (same submission_id, same ciphertext).
    expect(bodies).toHaveLength(2);
    expect(bodies[0]).toBe(bodies[1]);
    expect(JSON.parse(bodies[0]!).ciphertext).toBe(outer.ciphertext);
  });
});
