import { describe, expect, it } from "vitest";

import type { JsonObject } from "../../state/state";
import { consentText, consentTextDigest } from "./consent";
import {
  SUBMISSION_VERSION,
  TAGS_MAX_ITEMS,
  TEXT_MAX_LENGTH,
  advisoryIssues,
  buildPayload,
  canSubmit,
  fieldValue,
  formModel,
} from "./model";

const template: JsonObject = {
  schema_version: "0.1.0",
  template_id: "research",
  kinds: [
    {
      id: "person",
      label: "Person",
      attributes: [
        { id: "display_name", type: "text", required: true, default_visibility: "group" },
        { id: "affiliation", type: "tags" },
        { id: "capacity", type: "number" },
        { id: "status", type: "enum", values: ["active", "emeritus"] },
      ],
    },
  ],
};

function attrs() {
  return formModel(template).kinds[0]!.attributes;
}

describe("formModel (P3.6 - the template's kinds[].attributes[] consumed, R2)", () => {
  it("extracts kinds, attribute types, required flags, and enum values", () => {
    const model = formModel(template);
    expect(model.kinds).toHaveLength(1);
    const person = model.kinds[0]!;
    expect(person.label).toBe("Person");
    expect(person.attributes.map((a) => a.id)).toEqual([
      "display_name",
      "affiliation",
      "capacity",
      "status",
    ]);
    expect(person.attributes[0]!.required).toBe(true);
    expect(person.attributes[0]!.defaultVisibility).toBe("group");
    expect(person.attributes[3]!.values).toEqual(["active", "emeritus"]);
  });

  it("skips malformed kinds and attributes instead of throwing", () => {
    const model = formModel({
      kinds: [null, 5, { label: "no id" }, { id: "ok", attributes: [null, { type: "text" }] }],
    } as unknown as JsonObject);
    expect(model.kinds).toHaveLength(1);
    expect(model.kinds[0]!.attributes).toHaveLength(0);
  });
});

describe("fieldValue typed conversion", () => {
  it("converts by attribute type and omits empty inputs", () => {
    const [name, tags, capacity] = attrs();
    expect(fieldValue(name!, "  Ada Example ")).toBe("Ada Example");
    expect(fieldValue(name!, "   ")).toBeUndefined();
    expect(fieldValue(tags!, "River Alliance\n , Basin Group,")).toEqual([
      "River Alliance",
      "Basin Group",
    ]);
    expect(fieldValue(capacity!, "12.5")).toBe(12.5);
    expect(fieldValue(capacity!, "not a number")).toBeUndefined();
  });
});

describe("advisoryIssues (advisory UX only; the core stays authoritative, I2)", () => {
  it("flags missing required, bad numbers, off-list enums, and caps", () => {
    const [name, tags, capacity, status] = attrs();
    expect(advisoryIssues(name!, "")).toEqual(["This field is required."]);
    expect(advisoryIssues(capacity!, "abc")).toEqual(["Enter a number."]);
    expect(advisoryIssues(status!, "unlisted")[0]).toContain("listed choices");
    expect(advisoryIssues(name!, "x".repeat(TEXT_MAX_LENGTH + 1))[0]).toContain(
      String(TEXT_MAX_LENGTH),
    );
    const tooMany = Array.from({ length: TAGS_MAX_ITEMS + 1 }, (_, i) => `t${i}`).join(",");
    expect(advisoryIssues(tags!, tooMany)[0]).toContain(String(TAGS_MAX_ITEMS));
    expect(advisoryIssues(name!, "Ada")).toEqual([]);
    expect(advisoryIssues(tags!, "")).toEqual([]);
  });
});

describe("buildPayload (the inner-payload shape the queue stages)", () => {
  it("emits the blueprint section-4 shape with the payload-carried kind", () => {
    const payload = buildPayload({
      kind: "person",
      fields: { display_name: "Ada Example", affiliation: "River Alliance", capacity: "" },
      attributes: attrs(),
      consent: {
        consent_text_digest: "digest",
        consent_affirmed: true,
        consent_affirmed_at: 5,
      },
      submissionId: "sub-1",
      formVersion: "form-0.1",
      capturedAtMs: 4,
    });
    expect(payload).toEqual({
      submission_version: SUBMISSION_VERSION,
      submission_id: "sub-1",
      form_version: "form-0.1",
      kind: "person",
      consent: {
        consent_text_digest: "digest",
        consent_affirmed: true,
        consent_affirmed_at: 5,
      },
      captured_at: 4,
      fields: {
        display_name: "Ada Example",
        affiliation: ["River Alliance"],
      },
    });
  });
});

describe("canSubmit (the D-030 structural consent gate)", () => {
  it("blocks with the checkbox unchecked even when every field is valid", () => {
    const fields = { display_name: "Ada Example" };
    expect(canSubmit(attrs(), fields, false)).toBe(false);
    expect(canSubmit(attrs(), fields, true)).toBe(true);
  });

  it("blocks on advisory issues", () => {
    expect(canSubmit(attrs(), { display_name: "" }, true)).toBe(false);
  });
});

describe("consent boilerplate (D-072.1 draft)", () => {
  it("keeps the corrected claims and the placeholder discipline", () => {
    const text = consentText();
    expect(text).toContain("only what you typed");
    expect(text).toContain("no longer be shown");
    expect(text).toContain("[REMOVAL CONTACT");
    expect(text).not.toContain("taken out of the network");
    expect(text).not.toMatch(/@/);
  });

  it("digest is stable sha-256 hex over the canonical text", async () => {
    const digest = await consentTextDigest();
    expect(digest).toMatch(/^[0-9a-f]{64}$/);
    expect(digest).toBe(await consentTextDigest());
  });
});
