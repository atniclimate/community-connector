import type { ThemeToken } from "./tokens";

export const DEFAULT_THEME_TOKENS = {
  // bg.center is the ATNI design system's mandated dark-ground doctrine
  // (digest section 2, "Black BG"), applied here rather than in the renderer
  // so the OKLCH/CVD pipeline (derive.ts) still runs against it (CS-05,
  // D-101/D-102). bg.edge is a shifted-lightness step off the same hue, not
  // pure black, per the digest's own rule.
  "bg.center": { hex: "#010b13", source: "default" },
  "bg.edge": { hex: "#00040a", source: "default" },
  "surface.panel": { hex: "#151b26", source: "default" },
  "surface.scrim": { hex: "#080b11", source: "default" },
  // Digest section 2 "Text on Dark" - not pure white, used for presenter
  // captions and general body-on-dark text (CS-05).
  "text.primary": { hex: "#e8ecf0", source: "default" },
  "text.secondary": { hex: "#b5c0d0", source: "default" },
  "text.onAccent": { hex: "#081018", source: "default" },
  "accent.primary": { hex: "#e8b87c", source: "default" },
  "accent.focusRing": { hex: "#ffd38a", source: "default" },
  "label.text": { hex: "#e6edf7", source: "default" },
  "label.outline": { hex: "#06080d", source: "default" },
  "kind.default-1.base": { hex: "#7cc8e8", source: "default" },
  "kind.default-2.base": { hex: "#7ce8b2", source: "default" },
  "kind.default-3.base": { hex: "#e8a87c", source: "default" },
  "kind.default-4.base": { hex: "#e8d47c", source: "default" },
  "kind.default-5.base": { hex: "#9c8ce8", source: "default" },
  "kind.default-6.base": { hex: "#e87ca6", source: "default" },
  "kind.default-7.base": { hex: "#b8e87c", source: "default" },
  "kind.default-8.base": { hex: "#7ce8dc", source: "default" },
} as const satisfies Readonly<Record<string, ThemeToken>>;
