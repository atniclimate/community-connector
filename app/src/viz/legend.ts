import type { AppState, ShapeName } from "../state/state";
import type { Store } from "../state/store";
import type { Theme, ThemeReport } from "../theme/tokens";
import { RENDER_COLORS } from "./config";

// Legend overlay (P1.6): rendered from the live derived theme and the
// permission-filtered projection already held by the store (I2: nothing here
// computes visibility). Language quarantine (D-044.4): every instructional
// string this module renders is minimal and covered by the literal review
// marker below, which is always visible in the open panel.
export const REVIEW_MARKER = "DRAFT - PENDING HUMAN REVIEW (D-023)";
export const ADJUSTED_BADGE = "adjusted for readability";
const TOGGLE_TEXT = "Legend";
const KINDS_HEADING = "Entities";
const EDGES_HEADING = "Connections";
const EDGE_ENCODING_NOTE = "Line color blends the two connected entity colors; stronger connections are more opaque.";
const EMPTY_NOTE = "Nothing to show yet.";

export type LegendKindRow = {
  readonly kindId: string;
  readonly label: string;
  readonly shape: ShapeName;
  readonly colorHex: string;
  readonly adjusted: boolean;
  readonly adjustedReason: string | null;
};

export type LegendEdgeRow = {
  readonly kindId: string;
  readonly label: string;
};

export type LegendModel = {
  readonly kinds: readonly LegendKindRow[];
  readonly edges: readonly LegendEdgeRow[];
};

export function humanizeEdgeKind(kindId: string): string {
  return kindId.replaceAll("_", " ").replaceAll("-", " ");
}

function adjustedReason(report: ThemeReport | null, tokenName: string): string | null {
  const matches = (report?.adjustments ?? []).filter((adjustment) => adjustment.token === tokenName);
  return matches.at(-1)?.reason ?? null;
}

/** Pure view model: node kinds from the live theme, edge kinds from the projection. */
export function buildLegendModel(state: AppState): LegendModel {
  const theme = state.theme.resolved;
  const kinds = Object.entries(state.data.kindMeta).map(([kindId, meta]) => {
    const tokenName = `kind.${kindId}.base`;
    const token = theme?.tokens[tokenName];
    const adjusted = token?.source === "adjusted";
    return {
      kindId,
      label: meta.label,
      shape: meta.shape,
      colorHex: token?.hex ?? RENDER_COLORS.fallbackKind,
      adjusted,
      adjustedReason: adjusted ? adjustedReason(state.theme.report, tokenName) : null,
    };
  });
  const edgeKindIds = new Set<string>();
  for (const edge of state.data.projection?.edges ?? []) {
    if (typeof edge.kind === "string") {
      edgeKindIds.add(edge.kind);
    }
  }
  const edges = [...edgeKindIds]
    .sort((left, right) => left.localeCompare(right))
    .map((kindId) => ({ kindId, label: humanizeEdgeKind(kindId) }));
  return { kinds, edges };
}

function legendHex(theme: Theme | null, key: string, fallback: string): string {
  return theme?.tokens[key]?.hex ?? fallback;
}

// Structural styles only; colors come from the live theme at render time.
// No transitions or animations exist here, so the overlay is inert under
// prefers-reduced-motion by construction (I9).
const LEGEND_CSS = `
.cn-legend {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 10;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  max-width: min(280px, calc(100% - 24px));
  font-size: 13px;
  line-height: 1.45;
}
.cn-legend-toggle {
  font: inherit;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
}
.cn-legend-toggle:focus-visible {
  outline: 2px solid var(--cn-accent-focusRing, #ffd38a);
  outline-offset: 2px;
}
.cn-legend-panel {
  box-sizing: border-box;
  width: 100%;
  max-height: min(60vh, 420px);
  overflow-y: auto;
  padding: 10px 12px;
  border-radius: 8px;
}
.cn-legend-heading {
  margin: 8px 0 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.cn-legend-heading:first-child {
  margin-top: 0;
}
.cn-legend ul {
  margin: 0;
  padding: 0;
  list-style: none;
}
.cn-legend li {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 6px;
  margin: 2px 0;
}
.cn-legend-swatch {
  flex: none;
  align-self: center;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 1px solid rgb(255 255 255 / 0.25);
}
.cn-legend-shape {
  font-size: 11px;
}
.cn-legend-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid currentColor;
}
.cn-legend-note {
  margin: 4px 0 0;
  font-size: 11px;
}
.cn-legend-marker {
  margin: 8px 0 0;
  font-size: 10px;
  opacity: 0.8;
}
`;

let instanceCounter = 0;

type LegendElements = {
  readonly root: HTMLElement;
  readonly toggle: HTMLButtonElement;
  readonly panel: HTMLDivElement;
};

function createElements(panelId: string): LegendElements {
  const root = document.createElement("section");
  root.className = "cn-legend";
  root.setAttribute("aria-label", TOGGLE_TEXT);
  const style = document.createElement("style");
  style.textContent = LEGEND_CSS;
  const toggle = document.createElement("button");
  toggle.type = "button";
  toggle.className = "cn-legend-toggle";
  toggle.textContent = TOGGLE_TEXT;
  toggle.setAttribute("aria-controls", panelId);
  const panel = document.createElement("div");
  panel.className = "cn-legend-panel";
  panel.id = panelId;
  root.append(style, toggle, panel);
  return { root, toggle, panel };
}

function appendHeading(panel: HTMLElement, text: string): void {
  const heading = document.createElement("p");
  heading.className = "cn-legend-heading";
  heading.textContent = text;
  panel.append(heading);
}

function appendKindRow(list: HTMLElement, row: LegendKindRow, secondaryHex: string): void {
  const item = document.createElement("li");
  const swatch = document.createElement("span");
  swatch.className = "cn-legend-swatch";
  swatch.style.background = row.colorHex;
  swatch.setAttribute("aria-hidden", "true");
  const label = document.createElement("span");
  label.textContent = row.label;
  const shape = document.createElement("span");
  shape.className = "cn-legend-shape";
  shape.style.color = secondaryHex;
  shape.textContent = row.shape;
  item.append(swatch, label, shape);
  if (row.adjusted) {
    const badge = document.createElement("span");
    badge.className = "cn-legend-badge";
    badge.style.color = secondaryHex;
    badge.textContent = ADJUSTED_BADGE;
    if (row.adjustedReason !== null) {
      badge.title = row.adjustedReason;
    }
    item.append(badge);
  }
  list.append(item);
}

function appendNote(panel: HTMLElement, className: string, text: string, colorHex: string): void {
  const note = document.createElement("p");
  note.className = className;
  note.textContent = text;
  note.style.color = colorHex;
  panel.append(note);
}

function renderPanel(panel: HTMLElement, model: LegendModel, theme: Theme | null): void {
  panel.replaceChildren();
  const secondaryHex = legendHex(theme, "text.secondary", "#b5c0d0");
  if (model.kinds.length > 0) {
    appendHeading(panel, KINDS_HEADING);
    const kindList = document.createElement("ul");
    for (const row of model.kinds) {
      appendKindRow(kindList, row, secondaryHex);
    }
    panel.append(kindList);
  }
  if (model.edges.length > 0) {
    appendHeading(panel, EDGES_HEADING);
    const edgeList = document.createElement("ul");
    for (const row of model.edges) {
      const item = document.createElement("li");
      item.textContent = row.label;
      edgeList.append(item);
    }
    panel.append(edgeList);
    appendNote(panel, "cn-legend-note", EDGE_ENCODING_NOTE, secondaryHex);
  }
  if (model.kinds.length === 0 && model.edges.length === 0) {
    appendNote(panel, "cn-legend-note", EMPTY_NOTE, secondaryHex);
  }
  appendNote(panel, "cn-legend-marker", REVIEW_MARKER, secondaryHex);
}

function applyChromeColors(elements: LegendElements, theme: Theme | null): void {
  const panelBg = legendHex(theme, "surface.panel", "#151b26");
  const textHex = legendHex(theme, "text.primary", "#e6edf7");
  elements.toggle.style.background = panelBg;
  elements.toggle.style.color = textHex;
  elements.toggle.style.border = "1px solid rgb(255 255 255 / 0.15)";
  elements.panel.style.background = panelBg;
  elements.panel.style.color = textHex;
  elements.panel.style.border = "1px solid rgb(255 255 255 / 0.1)";
}

/**
 * Mounts the legend as a DOM overlay inside `container` (positioned relative
 * if it was static) and returns an unmount function. Open/closed state lives
 * in the app state machine (`ui.legendOpen`, toggled via `legendToggled`),
 * never locally (I4).
 */
export function mountLegend(container: HTMLElement, store: Store): () => void {
  instanceCounter += 1;
  const elements = createElements(`cn-legend-panel-${instanceCounter}`);
  if (getComputedStyle(container).position === "static") {
    container.style.position = "relative";
  }
  const onToggle = (): void => store.dispatch({ kind: "legendToggled" });
  elements.toggle.addEventListener("click", onToggle);
  let renderKey = "";
  const render = (): void => {
    const state = store.getState();
    const model = buildLegendModel(state);
    const open = state.ui.legendOpen;
    const key = JSON.stringify({ open, model, themeKey: state.theme.resolved?.tokens ?? null });
    if (key === renderKey) {
      return;
    }
    renderKey = key;
    elements.toggle.setAttribute("aria-expanded", String(open));
    elements.panel.hidden = !open;
    applyChromeColors(elements, state.theme.resolved);
    if (open) {
      renderPanel(elements.panel, model, state.theme.resolved);
    }
  };
  const unsubscribe = store.subscribe(render);
  container.append(elements.root);
  render();
  return () => {
    unsubscribe();
    elements.toggle.removeEventListener("click", onToggle);
    elements.root.remove();
  };
}
