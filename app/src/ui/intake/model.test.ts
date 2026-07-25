import { describe, expect, it } from "vitest";

import type { IntakeRecordSummaryDto, JsonObject } from "../../state/state";
import { dashboard, dedupKeys, nearDupAttrSets, nearDupRequest, reviewable } from "./model";

function summary(overrides: Partial<IntakeRecordSummaryDto>): IntakeRecordSummaryDto {
  return {
    recordId: "rec-1",
    reviewState: "pending",
    decisionGeneration: 0,
    payloadDigest: "rc",
    payloadHash: "ph",
    stagedAtMs: 1_000,
    payload: { submission_id: "sub-1", kind: "person", fields: {} },
    record: {},
    ...overrides,
  };
}

const template: JsonObject = {
  kinds: [
    {
      id: "person",
      label: "Person",
      attributes: [
        { id: "display_name", type: "text", required: true },
        { id: "bio", type: "text" },
        { id: "affiliation", type: "tags" },
      ],
    },
  ],
};

describe("dashboard (I12 visibility surface)", () => {
  it("counts states, tracks oldest pending age, flags unreadable", () => {
    const records = [
      summary({ recordId: "a", stagedAtMs: 1_000 }),
      summary({ recordId: "b", stagedAtMs: 5_000 }),
      summary({ recordId: "c", reviewState: "approved" }),
      summary({ recordId: "d", reviewState: null }),
    ];
    const view = dashboard(records, 2, 61_000);
    expect(view.countsByState).toEqual({ pending: 2, approved: 1 });
    expect(view.oldestPendingAgeMs).toBe(60_000);
    expect(view.unreadable).toBe(1);
    expect(view.pendingDecisionFiles).toBe(2);
  });

  it("reports null age with no pending records", () => {
    expect(dashboard([summary({ reviewState: "rejected" })], 0, 10).oldestPendingAgeMs).toBeNull();
  });
});

describe("nearDup helpers (template-driven, self-excluding)", () => {
  it("derives name attrs from required text and affiliation from tags", () => {
    expect(nearDupAttrSets(template, "person")).toEqual({
      nameAttrs: ["display_name"],
      affiliationAttrs: ["affiliation"],
    });
    expect(nearDupAttrSets(template, "unknown")).toEqual({
      nameAttrs: [],
      affiliationAttrs: [],
    });
  });

  it("builds the request over OTHER queue payloads only", () => {
    const target = summary({ recordId: "rec-1" });
    const other = summary({ recordId: "rec-2" });
    const request = nearDupRequest(template, target, [target, other]);
    expect(request["queue_payloads"]).toEqual([["rec-2", other.payload]]);
    expect(request["name_attrs"]).toEqual(["display_name"]);
  });
});

describe("dedupKeys (semantic keys of the other records)", () => {
  it("excludes the record itself and carries submission_id + payload_hash", () => {
    const target = summary({ recordId: "rec-1" });
    const other = summary({ recordId: "rec-2", payloadHash: "other-ph" });
    expect(dedupKeys(target, [target, other])).toEqual([
      { submission_id: "sub-1", payload_hash: "other-ph" },
    ]);
  });
});

describe("reviewable (the wizard refusal rule)", () => {
  it("refuses approved_intent and unreadable sidecars", () => {
    expect(reviewable(summary({}))).toBe(true);
    expect(reviewable(summary({ reviewState: "failed" }))).toBe(true);
    expect(reviewable(summary({ reviewState: "approved_intent" }))).toBe(false);
    expect(reviewable(summary({ reviewState: null }))).toBe(false);
  });
});
