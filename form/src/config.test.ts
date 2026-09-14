import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  ALLOWED_KINDS,
  FIELD_HELP_BY_TEMPLATE,
  FRIENDLY_LABELS_BY_TEMPLATE,
  QUESTION_ORDER_BY_TEMPLATE,
  TEMPLATE,
  TEMPLATE_ID,
  orderAttributes,
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

// The nine convention questions, verbatim from
// docs/design/intake-questions-2026-09-14.md (the human's wording).
const ATNI_QUESTIONS: readonly (readonly [string, string, string])[] = [
  ["display_name", "What is your Name?", "How do you prefer to be addressed?"],
  [
    "roles",
    "What do you Do?",
    "Share as many roles, positions, or areas of responsibility as feel right; a single title rarely tells the whole story...",
  ],
  [
    "origins",
    "Where are you from?",
    "The Tribe(s), Places, Communities, and Organizations that bring you here...",
  ],
  [
    "areas_of_interest",
    "What is important to you?",
    "The issues, committees, and areas you devote your time, energy, and thinking to... (e.g. climate, healthcare, human rights, etc.)",
  ],
  [
    "specialties",
    "What are you good at?",
    "The things people come to you for, whether or not they show up in a job description...",
  ],
  [
    "connections",
    "Who are you connected with? (Optional)",
    "The people, communities, and organizations you carry with you; the ones that stay on your mind when you think about this work...",
  ],
  [
    "seeking",
    "What are you hoping to find?",
    "The connection you came here looking for; a collaborator, a conversation, someone doing similar work, or something you haven't found yet...",
  ],
  [
    "offering",
    "What are you here to share?",
    "Your presence matters. The Knowledge, experience, opportunities, or gifts you bring into this room...",
  ],
  [
    "committee_memberships",
    "What committees will you attend?",
    "ATNI committees, working groups, or sessions you plan to participate in...",
  ],
];

function atniPerson() {
  const person = formModel(atni).kinds.find((k) => k.id === "person");
  expect(person).toBeDefined();
  return person!;
}

describe("per-template question order", () => {
  const order = QUESTION_ORDER_BY_TEMPLATE["atni-convention"]?.["person"];

  it("lists the nine convention questions in the human's order", () => {
    expect(order).toEqual(ATNI_QUESTIONS.map(([id]) => id));
  });

  it("names only attribute ids the template on disk defines, each once", () => {
    const ids = new Set(atniPerson().attributes.map((a) => a.id));
    for (const id of order ?? []) {
      expect(ids.has(id), `order id ${id}`).toBe(true);
    }
    expect(new Set(order).size).toBe(order?.length);
  });

  it("puts the required name question first and nothing else is required", () => {
    const ordered = orderAttributes(atniPerson().attributes, order);
    expect(ordered[0]?.id).toBe("display_name");
    expect(ordered[0]?.required).toBe(true);
    expect(ordered.slice(1).every((a) => !a.required)).toBe(true);
    // Every question after the name is a tags field (one per line).
    expect(ordered.slice(1).every((a) => a.attrType === "tags")).toBe(true);
  });

  it("drops the template attributes the form does not ask", () => {
    const ordered = orderAttributes(atniPerson().attributes, order).map((a) => a.id);
    for (const dropped of [
      "tribe",
      "role",
      "events_of_interest",
      "contact_email",
      "contact_preference",
      "organization_affiliations",
    ]) {
      expect(ordered, dropped).not.toContain(dropped);
    }
    expect(ordered).toHaveLength(9);
  });

  it("gives every listed attribute a label and help text, in the human's wording", () => {
    const labels = FRIENDLY_LABELS_BY_TEMPLATE["atni-convention"] ?? {};
    const help = FIELD_HELP_BY_TEMPLATE["atni-convention"] ?? {};
    for (const [id, label, helpText] of ATNI_QUESTIONS) {
      expect(labels[id], `label for ${id}`).toBe(label);
      expect(help[id], `help for ${id}`).toBe(helpText);
    }
    // And nothing beyond the nine (a label for an unasked attribute is dead text).
    expect(Object.keys(labels).sort()).toEqual([...(order ?? [])].sort());
    expect(Object.keys(help).sort()).toEqual([...(order ?? [])].sort());
  });

  it("throws on a listed attribute the template does not define (never a silent skip)", () => {
    const attributes = atniPerson().attributes;
    expect(() => orderAttributes(attributes, ["display_name", "rolez"])).toThrow(
      /unknown attribute "rolez"/,
    );
    expect(() => orderAttributes(attributes, [...(order ?? []), "tribal_nation"])).toThrow(
      /unknown attribute "tribal_nation"/,
    );
  });

  it("leaves a kind or template without an order untouched (today's behavior)", () => {
    const attributes = atniPerson().attributes;
    expect(orderAttributes(attributes, undefined)).toBe(attributes);
    expect(QUESTION_ORDER_BY_TEMPLATE["atni-convention"]?.["committee"]).toBeUndefined();
    expect(QUESTION_ORDER_BY_TEMPLATE["research-network"]).toBeUndefined();
    const researchPerson = formModel(research).kinds.find((k) => k.id === "person");
    expect(researchPerson).toBeDefined();
    expect(orderAttributes(researchPerson!.attributes, undefined)).toBe(researchPerson!.attributes);
  });
});

describe("per-template labels and help", () => {
  it("exposes the baked-in template id", () => {
    expect(TEMPLATE_ID).toBe(TEMPLATE["template_id"]);
  });

  it("keys every ATNI label and help entry to a real person attribute of that template", () => {
    const ids = new Set(atniPerson().attributes.map((a) => a.id));
    const labels = FRIENDLY_LABELS_BY_TEMPLATE["atni-convention"] ?? {};
    const help = FIELD_HELP_BY_TEMPLATE["atni-convention"] ?? {};
    for (const key of Object.keys(labels)) {
      expect(ids.has(key), `label key ${key}`).toBe(true);
    }
    for (const key of Object.keys(help)) {
      expect(ids.has(key), `help key ${key}`).toBe(true);
    }
  });

  it("carries no entry for templates that were not reviewed", () => {
    expect(FRIENDLY_LABELS_BY_TEMPLATE["research-network"]).toBeUndefined();
    expect(FIELD_HELP_BY_TEMPLATE["research-network"]).toBeUndefined();
  });
});
