import { contrastRatio, hexToOklch, oklchToHex, scaleChroma, setLightness, shiftLightness } from "./color";
import { solveContrast } from "./contrast";
import { enforceCvdDistance } from "./cvd";
import { DEFAULT_THEME_TOKENS } from "./defaults";
import type { DerivedTheme, GroupTemplateDto, ThemeAdjustment, ThemeReport, ThemeToken } from "./tokens";

type MutableTokenMap = Record<string, ThemeToken>;

const KIND_LIMIT = 8;

function isHex(value: string | undefined): value is string {
  return value !== undefined && /^#[0-9a-fA-F]{6}$/.test(value);
}

function defaultKindBase(index: number): string {
  const key = `kind.default-${(index % KIND_LIMIT) + 1}.base`;
  const defaults: Readonly<Record<string, ThemeToken>> = DEFAULT_THEME_TOKENS;
  const token = defaults[key];
  if (token === undefined) {
    throw new Error(`Missing default kind token: ${key}`);
  }
  return token.hex;
}

function token(hex: string, source: ThemeToken["source"]): ThemeToken {
  return { hex: hex.toLowerCase(), source };
}

function deriveKindTokens(base: string): Readonly<Record<"hover" | "dim" | "halo", string>> {
  return {
    hover: shiftLightness(base, 0.08),
    dim: scaleChroma(base, 0.35),
    halo: setLightness(base, 0.75),
  };
}

function collectKindBases(template: GroupTemplateDto): readonly { tokenName: string; hex: string; source: ThemeToken["source"] }[] {
  return template.kinds.slice(0, KIND_LIMIT).map((kind, index) => {
    const supplied = template.theme?.roles?.[kind.color_role];
    return {
      tokenName: `kind.${kind.id}.base`,
      hex: isHex(supplied) ? supplied : defaultKindBase(index),
      source: isHex(supplied) ? "template" : "default",
    };
  });
}

function mergeBaseTokens(template: GroupTemplateDto): MutableTokenMap {
  const tokens: MutableTokenMap = {};
  for (const [name, value] of Object.entries(DEFAULT_THEME_TOKENS)) {
    if (!name.startsWith("kind.default-")) {
      tokens[name] = value;
    }
  }
  for (const kindBase of collectKindBases(template)) {
    tokens[kindBase.tokenName] = token(kindBase.hex, kindBase.source);
    const variants = deriveKindTokens(kindBase.hex);
    tokens[kindBase.tokenName.replace(".base", ".hover")] = token(variants.hover, kindBase.source);
    tokens[kindBase.tokenName.replace(".base", ".dim")] = token(variants.dim, kindBase.source);
    tokens[kindBase.tokenName.replace(".base", ".halo")] = token(variants.halo, kindBase.source);
  }
  return tokens;
}

function applyAdjustment(tokens: MutableTokenMap, adjustment: ThemeAdjustment): void {
  tokens[adjustment.token] = token(adjustment.to, "adjusted");
}

function solveTextTokens(tokens: MutableTokenMap): readonly ThemeAdjustment[] {
  const pairs = [
    ["text.primary", "surface.panel"],
    ["text.secondary", "surface.panel"],
    ["text.onAccent", "accent.primary"],
  ] as const;
  return pairs.flatMap(([textToken, surfaceToken]) => {
    const text = tokens[textToken];
    const surface = tokens[surfaceToken];
    if (text === undefined || surface === undefined) {
      throw new Error(`Missing theme token pair: ${textToken} / ${surfaceToken}`);
    }
    const result = solveContrast(textToken, text.hex, surface.hex, 4.5, "WCAG text contrast floor");
    if (result.adjustment !== null) {
      applyAdjustment(tokens, result.adjustment);
      return [result.adjustment];
    }
    return [];
  });
}

function solveKindContrast(tokens: MutableTokenMap): readonly ThemeAdjustment[] {
  const background = tokens["bg.center"];
  if (background === undefined) {
    throw new Error("Missing theme token: bg.center");
  }
  return Object.entries(tokens).flatMap(([name, value]) => {
    if (!name.startsWith("kind.") || !name.endsWith(".base")) {
      return [];
    }
    const result = solveContrast(name, value.hex, background.hex, 3, "WCAG non-text contrast floor");
    if (result.adjustment !== null) {
      applyAdjustment(tokens, result.adjustment);
      const variants = deriveKindTokens(result.hex);
      tokens[name.replace(".base", ".hover")] = token(variants.hover, "adjusted");
      tokens[name.replace(".base", ".dim")] = token(variants.dim, "adjusted");
      tokens[name.replace(".base", ".halo")] = token(variants.halo, "adjusted");
      return [result.adjustment];
    }
    return [];
  });
}

function solveFocusRing(tokens: MutableTokenMap): readonly ThemeAdjustment[] {
  const ring = tokens["accent.focusRing"];
  const background = tokens["bg.center"];
  if (ring === undefined || background === undefined || contrastRatio(ring.hex, background.hex) >= 3) {
    return [];
  }
  const result = solveContrast("accent.focusRing", ring.hex, background.hex, 3, "WCAG focus indicator floor");
  if (result.adjustment === null) {
    return [];
  }
  applyAdjustment(tokens, result.adjustment);
  return [result.adjustment];
}

function solveCvd(tokens: MutableTokenMap): Pick<ThemeReport, "adjustments" | "warnings"> {
  const kindColors = Object.entries(tokens)
    .filter(([name]) => name.startsWith("kind.") && name.endsWith(".base"))
    .map(([tokenName, value]) => ({ token: tokenName, hex: value.hex }));
  const result = enforceCvdDistance(kindColors);
  for (const color of result.colors) {
    const previous = tokens[color.token];
    if (previous !== undefined && previous.hex !== color.hex) {
      tokens[color.token] = token(color.hex, "adjusted");
      const variants = deriveKindTokens(color.hex);
      tokens[color.token.replace(".base", ".hover")] = token(variants.hover, "adjusted");
      tokens[color.token.replace(".base", ".dim")] = token(variants.dim, "adjusted");
      tokens[color.token.replace(".base", ".halo")] = token(variants.halo, "adjusted");
    }
  }
  return { adjustments: result.adjustments, warnings: result.warnings };
}

function validateSchemaVersion(template: GroupTemplateDto): void {
  const major = Number.parseInt(template.schema_version.split(".")[0] ?? "", 10);
  if (major !== 0) {
    throw new Error(`Unsupported group template schema major: ${template.schema_version}`);
  }
}

function kindLimitWarning(template: GroupTemplateDto): ThemeReport["warnings"] {
  if (template.kinds.length <= KIND_LIMIT) {
    return [];
  }
  return [
    {
      code: "theme.kind-limit",
      message: "Distinguishability is guaranteed for at most 8 simultaneous kinds.",
      tokens: template.kinds.slice(KIND_LIMIT).map((kind) => `kind.${kind.id}.base`),
    },
  ];
}

export function normalizeToRestingRamp(hex: string): string {
  const color = hexToOklch(hex);
  return oklchToHex({ l: color.l, c: color.c, h: color.h });
}

export function deriveTheme(template: GroupTemplateDto): DerivedTheme {
  validateSchemaVersion(template);
  const tokens = mergeBaseTokens(template);
  const adjustments = [...solveTextTokens(tokens), ...solveKindContrast(tokens), ...solveFocusRing(tokens)];
  const cvd = solveCvd(tokens);
  const report: ThemeReport = {
    schema_version: "0.1.0",
    adjustments: [...adjustments, ...cvd.adjustments],
    warnings: [...kindLimitWarning(template), ...cvd.warnings],
  };
  return {
    theme: { schema_version: "0.1.0", tokens },
    report,
  };
}
