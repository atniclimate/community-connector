import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import type { JsonObject } from "../../state/state";
import { formModel } from "./model";

/**
 * Blueprint step 11: the synthetic intake demo submissions must stay
 * coherent with their group templates and the form-output shape - a
 * template edit that silently orphans a fixture fails here, not in a
 * live demo.
 */

const here = path.dirname(fileURLToPath(import.meta.url));
const fixtures = path.resolve(here, "../../../../fixtures");

type FixtureFile = {
  readonly group: string;
  readonly submissions: readonly JsonObject[];
};

function load(group: string): { file: FixtureFile; template: JsonObject } {
  const file = JSON.parse(
    readFileSync(path.join(fixtures, "intake", `${group}.submissions.json`), "utf8"),
  ) as FixtureFile;
  const template = JSON.parse(
    readFileSync(path.join(fixtures, "templates", `${group}.template.json`), "utf8"),
  ) as JsonObject;
  return { file, template };
}

for (const group of ["research-network", "fisheries-committee"]) {
  describe(`intake demo submissions: ${group}`, () => {
    const { file, template } = load(group);
    const model = formModel(template);

    it("carries the form-output shape with affirmed consent", () => {
      expect(file.submissions.length).toBeGreaterThan(0);
      for (const submission of file.submissions) {
        expect(submission["submission_version"]).toBe("0.1.0");
        expect(typeof submission["submission_id"]).toBe("string");
        expect(typeof submission["form_version"]).toBe("string");
        expect(typeof submission["captured_at"]).toBe("number");
        const consent = submission["consent"] as JsonObject;
        expect(consent["consent_affirmed"]).toBe(true);
        expect(typeof consent["consent_text_digest"]).toBe("string");
      }
    });

    it("targets template kinds and legal attribute ids and enum values", () => {
      for (const submission of file.submissions) {
        const kind = model.kinds.find((candidate) => candidate.id === submission["kind"]);
        expect(kind, `kind ${String(submission["kind"])} exists`).toBeDefined();
        const fields = submission["fields"] as JsonObject;
        for (const [attrId, value] of Object.entries(fields)) {
          const attr = kind?.attributes.find((candidate) => candidate.id === attrId);
          expect(attr, `${group}: attribute ${attrId} exists on ${String(submission["kind"])}`).toBeDefined();
          if (attr?.attrType === "enum" && typeof value === "string") {
            expect(attr.values).toContain(value);
          }
        }
      }
    });

    it("keeps exactly the deliberate demo defects and no accidental ones", () => {
      for (const submission of file.submissions) {
        const kind = model.kinds.find((candidate) => candidate.id === submission["kind"]);
        const fields = submission["fields"] as JsonObject;
        const missingRequired = (kind?.attributes ?? [])
          .filter((attr) => attr.required && fields[attr.id] === undefined)
          .map((attr) => attr.id);
        const note = submission["demo_note"];
        const isValidationDemo =
          typeof note === "string" && note.toLowerCase().includes("validation demo");
        if (isValidationDemo) {
          expect(missingRequired.length).toBeGreaterThan(0);
        } else {
          expect(missingRequired, `${String(submission["submission_id"])} complete`).toEqual([]);
        }
      }
    });
  });
}
