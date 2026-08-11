import { describe, expect, it } from "vitest";
import { bearer, dispatch, RELAY_ORIGIN } from "./helpers.js";

function req(path: string, method: string, headers: Record<string, string> = {}): Request {
  return new Request(`${RELAY_ORIGIN}${path}`, { method, headers });
}

describe("routing and method handling", () => {
  it("returns 405 for GET /submit (known path, unsupported method)", async () => {
    const res = await dispatch(req("/submit", "GET"));
    expect(res.status).toBe(405);
    expect(await res.json()).toEqual({ error: "method_not_allowed" });
    expect(res.headers.get("Allow")).toContain("POST");
  });

  it("returns 405 for PUT /blob/:id", async () => {
    const res = await dispatch(req("/blob/abc", "PUT", bearer()));
    expect(res.status).toBe(405);
  });

  it("returns 405 for POST /receipts", async () => {
    const res = await dispatch(req("/receipts", "POST", bearer()));
    expect(res.status).toBe(405);
  });

  it("returns a generic 404 for an unknown path, no crash or leak", async () => {
    const res = await dispatch(req("/totally/unknown", "GET"));
    expect(res.status).toBe(404);
    expect(await res.json()).toEqual({ error: "not_found" });
  });

  it("returns 404 for the /blob prefix with no id", async () => {
    const res = await dispatch(req("/blob/", "GET", bearer()));
    expect(res.status).toBe(404);
  });
});
