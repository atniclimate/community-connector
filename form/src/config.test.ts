import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  ALLOWED_KINDS,
  FIELD_HELP_BY_TEMPLATE,
  FRIENDLY_LABELS_BY_TEMPLATE,
  TEMPLATE,
  TEMPLATE_ID,
  parseKindList,
  restrictKinds,
} from "./config";
import type { JsonObject } from "./json";
import { formModel } from "./model";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

function loadTemplate(name: string): JsonObject {
  const file = path.join(repoRoot, "fixtures", "templates", name);
  return JSON.parse(readFileSync(file, "utf8")) as JsonObject;
}

const atni = loadTemplate("atni-convention.template.json");
const research = loadTemplate("research-network.template.json");

describe("CN_FORM_KINDS restriction", () => {
  it("parses a comma-separated list, trimming, dropping empties and duplicates", () => {
    expect(parseKindList("")).toEqual([]);
    expect(parseKindList("  ,  ")).toEqual([]);
    expect(parseKindList("person")).toEqual(["person"]);
    expect(parseKindList(" person, committee ,person,")).toEqual(["person", "committee"]);
  });

  it("leaves every kind in place when the list is empty (default build)", () => {
    const kinds = formModel(atni).kinds;
    expect(restrictKinds(kinds, [])).toBe(kinds);
    expect(restrictKinds(kinds, []).map((k) => k.id)).toEqual([
      "person",
      "committee",
      "organization",
    ]);
  });

  it("keeps only the allowed kinds, in template order (the ATNI build passes person)", () => {
    const kinds = formModel(atni).kinds;
    expect(restrictKinds(kinds, ["person"]).map((k) => k.id)).toEqual(["person"]);
    expect(restrictKinds(kinds, ["organization", "person"]).map((k) => k.id)).toEqual([
      "person",
      "organization",
    ]);
  });

  it("refuses an id the template does not define (never an empty form)", () => {
    const kinds = formModel(atni).kinds;
    expect(() => restrictKinds(kinds, ["persn"])).toThrow(/unknown kind "persn"/);
    expect(() => restrictKinds(formModel(research).kinds, ["committee"])).toThrow(
      /unknown kind "committee"/,
    );
  });

  it("defaults to no restriction in the test/dev build", () => {
    expect(ALLOWED_KINDS).toEqual([]);
  });
});

describe("per-template labels and help", () => {
  it("exposes the baked-in template id", () => {
    expect(TEMPLATE_ID).toBe(TEMPLATE["template_id"]);
  });

  it("keys every ATNI label and help entry to a real person attribute of that template", () => {
    const person = formModel(atni).kinds.find((k) => k.id === "person");
    expect(person).toBeDefined();
    const ids = new Set(person!.attributes.map((a) => a.id));
    const labels = FRIENDLY_LABELS_BY_TEMPLATE["atni-convention"] ?? {};
    const help = FIELD_HELP_BY_TEMPLATE["atni-convention"] ?? {};
    for (const key of Object.keys(labels)) {
      expect(ids.has(key), `label key ${key}`).toBe(true);
    }
    for (const key of Object.keys(help)) {
      expect(ids.has(key), `help key ${key}`).toBe(true);
    }
    // Every person attribute the ATNI form renders has a friendly label.
    for (const id of ids) {
      expect(labels[id], `label for ${id}`).toBeDefined();
    }
  });

  it("renders the eight ATNI labels exactly as reviewed", () => {
    expect(FRIENDLY_LABELS_BY_TEMPLATE["atni-convention"]).toEqual({
      display_name: "Name",
      tribe: "Tribal Nation or organization",
      role: "Role",
      areas_of_interest: "Priority areas",
      specialties: "Specialties",
      events_of_interest: "Events you plan to attend",
      contact_email: "Email",
      contact_preference: "How you prefer to be reached",
    });
    expect(FIELD_HELP_BY_TEMPLATE["atni-convention"]).toEqual({
      areas_of_interest: "One or two areas, one per line.",
    });
  });

  it("carries no entry for templates that were not reviewed", () => {
    expect(FRIENDLY_LABELS_BY_TEMPLATE["research-network"]).toBeUndefined();
    expect(FIELD_HELP_BY_TEMPLATE["research-network"]).toBeUndefined();
  });
});
