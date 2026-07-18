import { describe, expect, it } from "vitest";
import type { EntityDetailDto } from "../state/state";
import {
  DRAFT_REVIEW_MARKER,
  TIER_CODES,
  TIER_PLAIN,
  detailAttributeViews,
  extractProvenance,
  formatAttributeValue,
  humanizeAttrId,
  isTierCode,
} from "./format";

describe("formatAttributeValue", () => {
  it("formats every tagged cn-model value shape", () => {
    expect(formatAttributeValue({ type: "text", value: "Alder Willow 20" })).toBe("Alder Willow 20");
    expect(formatAttributeValue({ type: "enum", value: "active" })).toBe("active");
    expect(formatAttributeValue({ type: "number", value: 12.5 })).toBe("12.5");
    expect(formatAttributeValue({ type: "date", value: "2026-07-01" })).toBe("2026-07-01");
    expect(formatAttributeValue({ type: "tags", value: ["a", "b"] })).toBe("a, b");
    expect(formatAttributeValue({ type: "geo", value: { point: { lat: 45.5, lon: -122.6 } } })).toBe(
      "45.5, -122.6",
    );
    expect(formatAttributeValue({ type: "geo", value: { region: { name: "Synthetic Basin" } } })).toBe(
      "Synthetic Basin",
    );
    expect(
      formatAttributeValue({ type: "link", value: { url: "https://example.test/x", format: null } }),
    ).toBe("https://example.test/x");
    expect(formatAttributeValue({ type: "media", value: "00000000-0000-0000-0000-000000000001" })).toBe(
      "00000000-0000-0000-0000-000000000001",
    );
  });

  it("falls back to raw JSON for unknown shapes instead of dropping them", () => {
    expect(formatAttributeValue({ type: "mystery", value: 3 })).toBe('{"type":"mystery","value":3}');
    expect(formatAttributeValue(undefined)).toBe("");
  });
});

describe("tier language (D-032, D-023 quarantine)", () => {
  it("keeps the literal draft-review marker", () => {
    expect(DRAFT_REVIEW_MARKER).toBe("DRAFT - PENDING HUMAN REVIEW (D-023)");
  });

  it("has generic plain-language secondaries for all four TSDF codes", () => {
    for (const code of TIER_CODES) {
      expect(TIER_PLAIN[code].length).toBeGreaterThan(0);
      expect(isTierCode(code)).toBe(true);
    }
    expect(isTierCode("T4")).toBe(false);
    expect(isTierCode(1)).toBe(false);
  });
});

describe("detailAttributeViews", () => {
  it("carries value, visibility, and tier exactly as the payload does", () => {
    const detail: EntityDetailDto = {
      id: "00000000-0000-0000-0000-000000000001",
      kind: "person",
      owner_is_viewer: true,
      attributes: {
        display_name: {
          value: { type: "text", value: "Synthetic Person" },
          visibility: "Group",
          tier: "T1",
        },
        focus_area: { value: { type: "text", value: "Kelp" } },
      },
    };

    const views = detailAttributeViews(detail);
    const byId = new Map(views.map((view) => [view.id, view]));

    expect(byId.get("display_name")).toEqual({
      id: "display_name",
      label: "display name",
      valueText: "Synthetic Person",
      visibility: "Group",
      tier: "T1",
    });
    expect(byId.get("focus_area")?.visibility).toBeNull();
    expect(byId.get("focus_area")?.tier).toBeNull();
  });

  it("returns no rows when the payload has no attributes object", () => {
    expect(detailAttributeViews({ id: "x" })).toEqual([]);
  });

  it("humanizes attribute ids", () => {
    expect(humanizeAttrId("scientific_name")).toBe("scientific name");
    expect(humanizeAttrId("study-area")).toBe("study area");
  });
});

describe("extractProvenance (D-033: payload-driven, no viewer checks)", () => {
  it("returns null when the payload carries no provenance", () => {
    expect(extractProvenance({ id: "x" })).toEqual({ view: null, finding: null });
  });

  it("passes the core one-line summary through unchanged", () => {
    const result = extractProvenance({
      id: "x",
      provenance: { line: "Added by person p1 (2026-07-01)" },
    });
    expect(result).toEqual({
      view: { line: "Added by person p1 (2026-07-01)", chain: [] },
      finding: null,
    });
  });

  it("renders the custody chain only when the core includes it", () => {
    const envelope = {
      line: "Added by person 00000000-0000-0000-0000-0000000003e9 from synthetic-intake (timestamp 1782864000000)",
      custody: [
        {
          id: "00000000-0000-0000-0000-000000000201",
          action: "created",
          at: 1782864000000,
          actor: { human: "00000000-0000-0000-0000-0000000003e9" },
          note: null,
        },
        {
          id: "00000000-0000-0000-0000-000000000202",
          action: "corrected",
          at: 1782950400000,
          actor: { agent: { agent_id: "ingest-tool" } },
          note: "typo fix",
        },
      ],
    };

    const result = extractProvenance({ id: "x", provenance: envelope });
    if (result.view === null) {
      throw new Error("expected a provenance view");
    }
    expect(result.view.line).toBe(
      "Added by person 00000000-0000-0000-0000-0000000003e9 from synthetic-intake (timestamp 1782864000000)",
    );
    expect(result.view.chain).toEqual([
      "created by person 00000000-0000-0000-0000-0000000003e9 (2026-07-01)",
      "corrected by agent ingest-tool (2026-07-02) - typo fix",
    ]);

    const memberView = extractProvenance({
      id: "x",
      provenance: { ...envelope, custody: [] },
    });
    expect(memberView.view?.chain).toEqual([]);
  });

  it("returns a typed finding for malformed provenance and custody", () => {
    expect(extractProvenance({ id: "x", provenance: { custody: [] } }).finding).toEqual({
      code: "MalformedProvenance",
      message: "Entity detail provenance must contain a one-line summary",
    });
    expect(
      extractProvenance({ id: "x", provenance: { line: "Added", custody: "bad" } }).finding,
    ).toEqual({
      code: "MalformedProvenance",
      message: "Entity detail provenance custody must be an array",
    });
  });
});
