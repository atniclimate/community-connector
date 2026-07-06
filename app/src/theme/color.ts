export type Rgb = {
  readonly r: number;
  readonly g: number;
  readonly b: number;
};

export type Oklch = {
  readonly l: number;
  readonly c: number;
  readonly h: number;
};

export type Oklab = {
  readonly l: number;
  readonly a: number;
  readonly b: number;
};

const SRGB_TO_LMS = [
  [0.4122214708, 0.5363325363, 0.0514459929],
  [0.2119034982, 0.6806995451, 0.1073969566],
  [0.0883024619, 0.2817188376, 0.6299787005],
] as const;

const LMS_TO_OKLAB = [
  [0.2104542553, 0.793617785, -0.0040720468],
  [1.9779984951, -2.428592205, 0.4505937099],
  [0.0259040371, 0.7827717662, -0.808675766],
] as const;

const OKLAB_TO_LMS = [
  [1, 0.3963377774, 0.2158037573],
  [1, -0.1055613458, -0.0638541728],
  [1, -0.0894841775, -1.291485548],
] as const;

const LMS_TO_SRGB = [
  [4.0767416621, -3.3077115913, 0.2309699292],
  [-1.2684380046, 2.6097574011, -0.3413193965],
  [-0.0041960863, -0.7034186147, 1.707614701],
] as const;

function clamp(value: number, min = 0, max = 1): number {
  return Math.min(max, Math.max(min, value));
}

function linearize(channel: number): number {
  return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

function delinearize(channel: number): number {
  return channel <= 0.0031308 ? channel * 12.92 : 1.055 * channel ** (1 / 2.4) - 0.055;
}

function multiply(matrix: readonly (readonly number[])[], rgb: Rgb): Rgb {
  const [x, y, z] = matrix.map(
    (row) => (row[0] ?? 0) * rgb.r + (row[1] ?? 0) * rgb.g + (row[2] ?? 0) * rgb.b,
  );
  return { r: x ?? 0, g: y ?? 0, b: z ?? 0 };
}

export function hexToRgb(hex: string): Rgb {
  if (!/^#[0-9a-fA-F]{6}$/.test(hex)) {
    throw new Error(`Invalid hex color: ${hex}`);
  }
  return {
    r: Number.parseInt(hex.slice(1, 3), 16) / 255,
    g: Number.parseInt(hex.slice(3, 5), 16) / 255,
    b: Number.parseInt(hex.slice(5, 7), 16) / 255,
  };
}

export function rgbToHex(rgb: Rgb): string {
  const toByte = (value: number): string =>
    Math.round(clamp(value) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${toByte(rgb.r)}${toByte(rgb.g)}${toByte(rgb.b)}`;
}

export function rgbToOklab(rgb: Rgb): Oklab {
  const linear = { r: linearize(rgb.r), g: linearize(rgb.g), b: linearize(rgb.b) };
  const lms = multiply(SRGB_TO_LMS, linear);
  const rooted = { r: Math.cbrt(lms.r), g: Math.cbrt(lms.g), b: Math.cbrt(lms.b) };
  const lab = multiply(LMS_TO_OKLAB, rooted);
  return { l: lab.r, a: lab.g, b: lab.b };
}

export function oklabToRgb(lab: Oklab): Rgb {
  const lmsRoot = multiply(OKLAB_TO_LMS, { r: lab.l, g: lab.a, b: lab.b });
  const lms = { r: lmsRoot.r ** 3, g: lmsRoot.g ** 3, b: lmsRoot.b ** 3 };
  const linear = multiply(LMS_TO_SRGB, lms);
  return {
    r: clamp(delinearize(linear.r)),
    g: clamp(delinearize(linear.g)),
    b: clamp(delinearize(linear.b)),
  };
}

export function hexToOklch(hex: string): Oklch {
  const lab = rgbToOklab(hexToRgb(hex));
  const hue = Math.atan2(lab.b, lab.a) * (180 / Math.PI);
  return { l: lab.l, c: Math.hypot(lab.a, lab.b), h: hue < 0 ? hue + 360 : hue };
}

export function oklchToHex(color: Oklch): string {
  const radians = color.h * (Math.PI / 180);
  return rgbToHex(oklabToRgb({ l: color.l, a: color.c * Math.cos(radians), b: color.c * Math.sin(radians) }));
}

export function relativeLuminance(hex: string): number {
  const rgb = hexToRgb(hex);
  return 0.2126 * linearize(rgb.r) + 0.7152 * linearize(rgb.g) + 0.0722 * linearize(rgb.b);
}

export function contrastRatio(a: string, b: string): number {
  const first = relativeLuminance(a);
  const second = relativeLuminance(b);
  const lighter = Math.max(first, second);
  const darker = Math.min(first, second);
  return (lighter + 0.05) / (darker + 0.05);
}

export function hueDistance(a: number, b: number): number {
  const delta = Math.abs(a - b) % 360;
  return Math.min(delta, 360 - delta);
}

export function shiftLightness(hex: string, delta: number): string {
  const color = hexToOklch(hex);
  return oklchToHex({ ...color, l: clamp(color.l + delta) });
}

export function setLightness(hex: string, l: number): string {
  const color = hexToOklch(hex);
  return oklchToHex({ ...color, l: clamp(l) });
}

export function scaleChroma(hex: string, scale: number): string {
  const color = hexToOklch(hex);
  return oklchToHex({ ...color, c: Math.max(0, color.c * scale) });
}

export function rotateHue(hex: string, degrees: number): string {
  const color = hexToOklch(hex);
  return oklchToHex({ ...color, h: (color.h + degrees + 360) % 360 });
}
