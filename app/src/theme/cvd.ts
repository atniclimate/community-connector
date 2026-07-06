import { hexToRgb, rgbToOklab, rotateHue, setLightness } from "./color";
import type { ThemeAdjustment, ThemeWarning } from "./tokens";

type KindColor = {
  readonly token: string;
  readonly hex: string;
};

type CvdResult = {
  readonly colors: readonly KindColor[];
  readonly adjustments: readonly ThemeAdjustment[];
  readonly warnings: readonly ThemeWarning[];
};

const DISTANCE_FLOOR = 0.085;
const MAX_ITERATIONS = 8;

const SIMULATIONS = [
  [
    [0.367, 0.861, -0.228],
    [0.28, 0.673, 0.047],
    [-0.012, 0.043, 0.969],
  ],
  [
    [0.152, 1.053, -0.205],
    [0.115, 0.786, 0.099],
    [-0.004, -0.048, 1.052],
  ],
] as const;

function simulate(hex: string, matrix: readonly (readonly number[])[]): string {
  const rgb = hexToRgb(hex);
  const r = matrix[0]?.[0] ?? 1;
  const rg = matrix[0]?.[1] ?? 0;
  const rb = matrix[0]?.[2] ?? 0;
  const gr = matrix[1]?.[0] ?? 0;
  const g = matrix[1]?.[1] ?? 1;
  const gb = matrix[1]?.[2] ?? 0;
  const br = matrix[2]?.[0] ?? 0;
  const bg = matrix[2]?.[1] ?? 0;
  const b = matrix[2]?.[2] ?? 1;
  return `#${[r * rgb.r + rg * rgb.g + rb * rgb.b, gr * rgb.r + g * rgb.g + gb * rgb.b, br * rgb.r + bg * rgb.g + b * rgb.b]
    .map((channel) =>
      Math.round(Math.min(1, Math.max(0, channel)) * 255)
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
}

function colorDistance(a: string, b: string): number {
  return Math.min(
    ...SIMULATIONS.map((matrix) => {
      const first = rgbToOklab(hexToRgb(simulate(a, matrix)));
      const second = rgbToOklab(hexToRgb(simulate(b, matrix)));
      return Math.hypot(first.l - second.l, first.a - second.a, first.b - second.b);
    }),
  );
}

function firstFailingPair(colors: readonly KindColor[]): readonly [number, number] | null {
  for (let first = 0; first < colors.length; first += 1) {
    for (let second = first + 1; second < colors.length; second += 1) {
      const a = colors[first];
      const b = colors[second];
      if (a !== undefined && b !== undefined && colorDistance(a.hex, b.hex) < DISTANCE_FLOOR) {
        return [first, second];
      }
    }
  }
  return null;
}

function nudged(hex: string, iteration: number): string {
  const rotated = rotateHue(hex, iteration % 2 === 0 ? 12 : -12);
  return iteration < 4 ? rotated : setLightness(rotated, 0.72 + (iteration - 3) * 0.035);
}

export function enforceCvdDistance(input: readonly KindColor[]): CvdResult {
  let colors = [...input];
  const adjustments: ThemeAdjustment[] = [];
  const warnings: ThemeWarning[] = [];
  for (let iteration = 0; iteration < MAX_ITERATIONS; iteration += 1) {
    const pair = firstFailingPair(colors);
    if (pair === null) {
      return { colors, adjustments, warnings };
    }
    const later = colors[pair[1]];
    if (later === undefined) {
      break;
    }
    const nextHex = nudged(later.hex, iteration);
    colors = colors.map((color, index) => (index === pair[1] ? { ...color, hex: nextHex } : color));
    adjustments.push({
      token: later.token,
      from: later.hex,
      to: nextHex,
      reason: "CVD distance floor",
    });
  }
  const unresolved = firstFailingPair(colors);
  if (unresolved !== null) {
    const first = colors[unresolved[0]];
    const second = colors[unresolved[1]];
    warnings.push({
      code: "theme.cvd-distance-cap",
      message: "CVD distance enforcement reached the 8-iteration cap.",
      tokens: first !== undefined && second !== undefined ? [first.token, second.token] : [],
    });
  }
  return { colors, adjustments, warnings };
}
