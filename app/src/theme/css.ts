import type { Theme } from "./tokens";

function cssName(tokenName: string): string {
  return `--cn-${tokenName.replaceAll(".", "-").replaceAll("_", "-")}`;
}

export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement): void {
  for (const [name, token] of Object.entries(theme.tokens)) {
    root.style.setProperty(cssName(name), token.hex);
  }
}
