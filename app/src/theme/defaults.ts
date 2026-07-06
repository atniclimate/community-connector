import type { ThemeToken } from "./tokens";

export const DEFAULT_THEME_TOKENS = {
  "bg.center": { hex: "#0d1017", source: "default" },
  "bg.edge": { hex: "#06080d", source: "default" },
  "surface.panel": { hex: "#151b26", source: "default" },
  "surface.scrim": { hex: "#080b11", source: "default" },
  "text.primary": { hex: "#e6edf7", source: "default" },
  "text.secondary": { hex: "#b5c0d0", source: "default" },
  "text.onAccent": { hex: "#081018", source: "default" },
  "accent.primary": { hex: "#e8b87c", source: "default" },
  "accent.focusRing": { hex: "#ffd38a", source: "default" },
  "kind.default-1.base": { hex: "#7cc8e8", source: "default" },
  "kind.default-2.base": { hex: "#7ce8b2", source: "default" },
  "kind.default-3.base": { hex: "#e8a87c", source: "default" },
  "kind.default-4.base": { hex: "#e8d47c", source: "default" },
  "kind.default-5.base": { hex: "#9c8ce8", source: "default" },
  "kind.default-6.base": { hex: "#e87ca6", source: "default" },
  "kind.default-7.base": { hex: "#b8e87c", source: "default" },
  "kind.default-8.base": { hex: "#7ce8dc", source: "default" },
} as const satisfies Readonly<Record<string, ThemeToken>>;
