import Ajv from "ajv/dist/2020";
import { describe, expect, it } from "vitest";
import fisheriesJson from "../../../fixtures/templates/fisheries-committee.template.json?raw";
import researchJson from "../../../fixtures/templates/research-network.template.json?raw";
import themeSchemaJson from "../../../schemas/theme-tokens.schema.json?raw";
import { contrastRatio, hexToOklch, hexToRgb, hueDistance, oklchToHex, rgbToHex } from "./color";
import { preservesHue } from "./contrast";
import { deriveTheme } from "./derive";
import type { GroupTemplateDto } from "./tokens";

function fixture(name: string): GroupTemplateDto {
  return JSON.parse(name === "research-network.template.json" ? researchJson : fisheriesJson) as GroupTemplateDto;
}

function sabotageTemplate(): GroupTemplateDto {
  const narrow = ["#11161c", "#12171d", "#13181e", "#14191f", "#151a20", "#161b21"];
  const roles = Object.fromEntries(
    Array.from({ length: 6 }, (_, index) => [`kind-${index + 1}`, narrow[index] ?? "#11161c"]),
  ) as Readonly<Record<string, string>>;
  return {
    schema_version: "0.1.0",
    kinds: Array.from({ length: 6 }, (_, index) => ({
      id: `kind_${index + 1}`,
      color_role: `kind-${index + 1}`,
    })),
    theme: { roles },
  };
}

describe("theme color math", () => {
  it("round-trips sRGB through OKLCH within byte tolerance", () => {
    for (const hex of ["#000000", "#ffffff", "#7cc8e8", "#e8a87c", "#123456", "#abcdef"]) {
      const roundTrip = oklchToHex(hexToOklch(rgbToHex(hexToRgb(hex))));
      expect(Math.abs(Number.parseInt(roundTrip.slice(1, 3), 16) - Number.parseInt(hex.slice(1, 3), 16))).toBeLessThanOrEqual(1);
      expect(Math.abs(Number.parseInt(roundTrip.slice(3, 5), 16) - Number.parseInt(hex.slice(3, 5), 16))).toBeLessThanOrEqual(1);
      expect(Math.abs(Number.parseInt(roundTrip.slice(5, 7), 16) - Number.parseInt(hex.slice(5, 7), 16))).toBeLessThanOrEqual(1);
    }
  });

  it("matches known WCAG contrast values", () => {
    expect(contrastRatio("#ffffff", "#000000")).toBeCloseTo(21, 5);
    expect(contrastRatio("#777777", "#ffffff")).toBeCloseTo(4.48, 1);
  });
});

describe("theme derivation", () => {
  it("is deterministic for byte-identical input", () => {
    const template = fixture("research-network.template.json");
    expect(JSON.stringify(deriveTheme(template))).toBe(JSON.stringify(deriveTheme(template)));
  });

  it("derives the fisheries fixture with expected report outcomes", () => {
    const derived = deriveTheme(fixture("fisheries-committee.template.json"));

    expect(derived.report.adjustments.map((entry) => entry.token)).toEqual([
      "kind.need.base",
      "kind.need.base",
      "kind.need.base",
      "kind.need.base",
      "kind.need.base",
      "kind.skill_resource.base",
      "kind.skill_resource.base",
      "kind.skill_resource.base",
    ]);
    expect(derived.report.warnings.map((warning) => ({ code: warning.code, tokens: warning.tokens }))).toEqual([
      {
        code: "theme.cvd-distance-cap",
        tokens: ["kind.fishing_site.base", "kind.skill_resource.base"],
      },
    ]);
  });

  it("derives the research fixture with expected report outcomes", () => {
    const derived = deriveTheme(fixture("research-network.template.json"));

    expect(derived.report.adjustments.map((entry) => entry.token)).toEqual([
      "kind.study_area.base",
      "kind.publication.base",
      "kind.publication.base",
      "kind.publication.base",
      "kind.publication.base",
      "kind.publication.base",
      "kind.publication.base",
      "kind.publication.base",
    ]);
    expect(derived.report.warnings.map((warning) => ({ code: warning.code, tokens: warning.tokens }))).toEqual([
      {
        code: "theme.cvd-distance-cap",
        tokens: ["kind.publication.base", "kind.study_area.base"],
      },
    ]);
  });

  it("records adjustments and warnings for a hostile narrow palette", () => {
    const derived = deriveTheme(sabotageTemplate());
    const bases = Object.entries(derived.theme.tokens).filter(([name]) => name.startsWith("kind.") && name.endsWith(".base"));

    expect(derived.report.adjustments.length).toBeGreaterThan(0);
    expect(derived.report.warnings.some((warning) => warning.code === "theme.cvd-distance-cap")).toBe(true);
    for (const [, token] of bases) {
      expect(contrastRatio(token.hex, derived.theme.tokens["bg.center"]?.hex ?? "#000000")).toBeGreaterThanOrEqual(3);
    }
  });

  it("adjusts low-contrast text to the WCAG floor while preserving hue", () => {
    const template: GroupTemplateDto = {
      schema_version: "0.1.0",
      kinds: [{ id: "person", color_role: "kind-1" }],
      theme: { roles: { "kind-1": "#7cc8e8" } },
    };
    const derived = deriveTheme(template);
    const text = derived.theme.tokens["text.secondary"];
    const surface = derived.theme.tokens["surface.panel"];

    expect(text).toBeDefined();
    expect(surface).toBeDefined();
    expect(contrastRatio(text?.hex ?? "#000000", surface?.hex ?? "#ffffff")).toBeGreaterThanOrEqual(4.5);
    expect(preservesHue("#b5c0d0", text?.hex ?? "#000000", 2)).toBe(true);
    expect(hueDistance(hexToOklch("#b5c0d0").h, hexToOklch(text?.hex ?? "#000000").h)).toBeLessThan(2);
  });

  it("validates serialized themes against the schema and rejects unknown majors", () => {
    const schema = JSON.parse(themeSchemaJson);
    const ajv = new Ajv();
    const validate = ajv.compile(schema);
    const derived = deriveTheme(fixture("research-network.template.json"));
    const serialized = { schema_version: "0.1.0", theme: derived.theme, report: derived.report };

    expect(validate(serialized)).toBe(true);
    expect(validate({ ...serialized, schema_version: "2.0.0" })).toBe(false);
    expect(() => deriveTheme({ ...fixture("research-network.template.json"), schema_version: "2.0.0" })).toThrow(
      /Unsupported group template schema major/,
    );
  });
});
