import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import type { JsonObject } from "./json";
import { buildFields, fieldValue, formModel, type FormAttr } from "./model";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

function loadTemplate(name: string): JsonObject {
  const file = path.join(repoRoot, "fixtures", "templates", name);
  return JSON.parse(readFileSync(file, "utf8")) as JsonObject;
}

const research = loadTemplate("research-network.template.json");
const fisheries = loadTemplate("fisheries-committee.template.json");

function attrIds(template: JsonObject, kindId: string): string[] {
  const model = formModel(template);
  const kind = model.kinds.find((k) => k.id === kindId);
  return kind ? kind.attributes.map((a) => a.id) : [];
}

function attr(template: JsonObject, kindId: string, attrId: string): FormAttr {
  const model = formModel(template);
  const found = model.kinds.find((k) => k.id === kindId)?.attributes.find((a) => a.id === attrId);
  if (found === undefined) {
    throw new Error(`no attr ${kindId}.${attrId}`);
  }
  return found;
}

describe("formModel", () => {
  it("extracts all kinds from both fixture templates", () => {
    expect(formModel(research).kinds.map((k) => k.id)).toEqual([
      "person",
      "organization",
      "publication",
      "study_area",
      "species",
      "gathering",
    ]);
    expect(formModel(fisheries).kinds.map((k) => k.id)).toEqual([
      "person",
      "fishing_site",
      "need",
      "skill_resource",
      "project",
      "gathering",
    ]);
  });

  it("excludes media attributes (real fixtures exercise this)", () => {
    // research-network person has a `portrait` media attribute.
    expect(attrIds(research, "person")).not.toContain("portrait");
    // fisheries-committee person.family_canoe_photo and gathering.flyer are media.
    expect(attrIds(fisheries, "person")).not.toContain("family_canoe_photo");
    expect(attrIds(fisheries, "gathering")).not.toContain("flyer");

    // No attribute of any kind in either template is media-typed.
    for (const template of [research, fisheries]) {
      for (const kind of formModel(template).kinds) {
        for (const a of kind.attributes) {
          expect(a.attrType).not.toBe("media");
        }
      }
    }
  });

  it("keeps the non-media attributes and their types", () => {
    expect(attrIds(fisheries, "person")).toEqual([
      "display_name",
      "contact_email",
      "roles",
      "seasons_active",
    ]);
    expect(attr(fisheries, "person", "seasons_active").attrType).toBe("enum");
    expect(attr(fisheries, "person", "seasons_active").values).toContain("year-round");
    expect(attr(fisheries, "person", "display_name").required).toBe(true);
  });
});

describe("fieldValue conversions", () => {
  const text: FormAttr = { id: "t", attrType: "text", required: false, values: [], defaultVisibility: null };
  const num: FormAttr = { id: "n", attrType: "number", required: false, values: [], defaultVisibility: null };
  const tags: FormAttr = { id: "g", attrType: "tags", required: false, values: [], defaultVisibility: null };
  const geo: FormAttr = { id: "geo", attrType: "geo", required: false, values: [], defaultVisibility: null };

  it("keeps text as a trimmed string", () => {
    expect(fieldValue(text, "  hello  ")).toBe("hello");
  });

  it("parses numbers as JSON numbers", () => {
    expect(fieldValue(num, "42.5")).toBe(42.5);
    expect(fieldValue(num, "not-a-number")).toBeUndefined();
  });

  it("splits tags on newline/comma, trimming and dropping empties", () => {
    expect(fieldValue(tags, "a, b\nc,,")).toEqual(["a", "b", "c"]);
  });

  it("parses geo as {lat, lon} for a coordinate, else {name}", () => {
    expect(fieldValue(geo, "47.6, -122.3")).toEqual({ lat: 47.6, lon: -122.3 });
    expect(fieldValue(geo, "Winterberry River")).toEqual({ name: "Winterberry River" });
  });

  it("omits empty optional fields", () => {
    expect(fieldValue(text, "   ")).toBeUndefined();
  });
});

describe("buildFields", () => {
  it("assembles only non-empty typed fields", () => {
    const attrs = formModel(fisheries).kinds.find((k) => k.id === "person")!.attributes;
    const fields = buildFields(attrs, {
      display_name: "Synthetic Person",
      roles: "fisher\nsteward",
      seasons_active: "",
    });
    expect(fields).toEqual({
      display_name: "Synthetic Person",
      roles: ["fisher", "steward"],
    });
    // media never appears (family_canoe_photo was excluded from attrs).
    expect(Object.keys(fields)).not.toContain("family_canoe_photo");
  });
});
