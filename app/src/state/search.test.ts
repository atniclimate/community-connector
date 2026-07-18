import { describe, expect, it } from "vitest";
import { parseSearchHits } from "./search";

describe("parseSearchHits", () => {
  it("accepts a valid hit array", () => {
    const parsed = parseSearchHits([
      {
        entity: "00000000-0000-0000-0000-000000000001",
        kind: "person",
        matched_attr: "display_name",
        snippet: "Synthetic Person",
      },
    ]);

    if ("error" in parsed) {
      throw new Error(`expected hits, got error: ${parsed.error.message}`);
    }
    expect(parsed.hits).toHaveLength(1);
    expect(parsed.hits[0]?.matched_attr).toBe("display_name");
  });

  it("accepts an empty array as zero hits", () => {
    const parsed = parseSearchHits([]);

    expect("error" in parsed).toBe(false);
    if (!("error" in parsed)) {
      expect(parsed.hits).toHaveLength(0);
    }
  });

  it("rejects a non-array payload with a typed error", () => {
    const parsed = parseSearchHits({ hits: [] });

    if (!("error" in parsed)) {
      throw new Error("expected an error for a non-array payload");
    }
    expect(parsed.error.code).toBe("ProtocolError");
  });

  it("rejects malformed rows and names the offending index", () => {
    const parsed = parseSearchHits([
      { entity: "e1", kind: "person", matched_attr: "a", snippet: "s" },
      { entity: "e2", kind: "person", snippet: "missing matched_attr" },
    ]);

    if (!("error" in parsed)) {
      throw new Error("expected an error for a malformed row");
    }
    expect(parsed.error.message).toContain("index 1");
  });
});
