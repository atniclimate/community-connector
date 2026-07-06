import { contrastRatio, hexToOklch, hueDistance, oklchToHex } from "./color";
import type { ThemeAdjustment } from "./tokens";

const LIGHTNESS_STEP = 0.005;
const MAX_STEPS = 200;

export type ContrastResult = {
  readonly hex: string;
  readonly adjustment: ThemeAdjustment | null;
};

function candidateAt(hex: string, delta: number): string {
  const color = hexToOklch(hex);
  return oklchToHex({ ...color, l: Math.min(1, Math.max(0, color.l + delta)) });
}

function bestDirection(hex: string, surface: string, floor: number): 1 | -1 {
  const lighter = candidateAt(hex, LIGHTNESS_STEP);
  const darker = candidateAt(hex, -LIGHTNESS_STEP);
  if (contrastRatio(lighter, surface) >= floor) {
    return 1;
  }
  if (contrastRatio(darker, surface) >= floor) {
    return -1;
  }
  return contrastRatio(lighter, surface) >= contrastRatio(darker, surface) ? 1 : -1;
}

export function solveContrast(
  token: string,
  hex: string,
  surface: string,
  floor: number,
  reason: string,
): ContrastResult {
  if (contrastRatio(hex, surface) >= floor) {
    return { hex, adjustment: null };
  }
  const direction = bestDirection(hex, surface, floor);
  let solved = hex;
  for (let step = 1; step <= MAX_STEPS; step += 1) {
    solved = candidateAt(hex, direction * step * LIGHTNESS_STEP);
    if (contrastRatio(solved, surface) >= floor) {
      return { hex: solved, adjustment: { token, from: hex, to: solved, reason } };
    }
  }
  return { hex: solved, adjustment: { token, from: hex, to: solved, reason } };
}

export function preservesHue(before: string, after: string, maxDelta: number): boolean {
  return hueDistance(hexToOklch(before).h, hexToOklch(after).h) < maxDelta;
}
