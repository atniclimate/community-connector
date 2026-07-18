import type { Store } from "../state/store";
import type { KindMeta, ProjectionDto } from "../state/state";
import { degreeByEntityId, entityLabel, projectedEdges, projectedEntities } from "../viz/projection";
import { el, uiId } from "./dom";
import "./ui.css";

/**
 * P1.7 flat list/table reading view (D-035 accessibility down-payment). Rows
 * are derived from the SAME projection adapter the 3D scene consumes
 * (`projectedEntities`), so the flat view and the scene agree on the visible
 * entity set for the viewer scope by construction; the vitest in flat.test.ts
 * pins the agreement against a real fixture projection.
 */

export type FlatRow = {
  readonly id: string;
  readonly kind: string;
  readonly label: string;
  readonly kindLabel: string;
  readonly degree: number;
};

export function flatRows(
  projection: ProjectionDto | null,
  kindMeta: Readonly<Record<string, KindMeta>>,
): readonly FlatRow[] {
  const entities = projectedEntities(projection);
  const degrees = degreeByEntityId(entities, projectedEdges(projection));
  const rows = entities.map((entity) => ({
    id: entity.id,
    kind: entity.kind,
    label: entityLabel(entity, kindMeta),
    kindLabel: kindMeta[entity.kind]?.label ?? entity.kind,
    degree: degrees.get(entity.id) ?? 0,
  }));
  return rows.sort((a, b) => a.label.localeCompare(b.label) || a.id.localeCompare(b.id));
}

export type FlatDeps = {
  readonly store: Store;
};

export function mountFlatProjection(container: HTMLElement, deps: FlatDeps): () => void {
  const { store } = deps;
  const headingId = uiId("cn-flat-heading");
  const heading = el("h2", { className: "cn-flat-title", text: "Entity list", attrs: { id: headingId } });
  const count = el("p", { className: "cn-flat-count", attrs: { "aria-live": "polite" } });
  const tableWrap = el("div", { className: "cn-flat-scroll" });
  const root = el(
    "section",
    { className: "cn-flat", attrs: { "aria-labelledby": headingId } },
    [heading, count, tableWrap],
  );
  container.replaceChildren(root);

  let renderedProjection: ProjectionDto | null = null;
  let renderedKindMeta: Readonly<Record<string, KindMeta>> | null = null;

  function buildTable(rows: readonly FlatRow[]): HTMLElement {
    const head = el("thead", {}, [
      el("tr", {}, [
        el("th", { text: "Name", attrs: { scope: "col" } }),
        el("th", { text: "Kind", attrs: { scope: "col" } }),
        el("th", { text: "Connections", attrs: { scope: "col" } }),
      ]),
    ]);
    const body = el("tbody");
    for (const row of rows) {
      const open = el("button", {
        className: "cn-flat-open",
        text: row.label,
        attrs: { type: "button", "aria-label": `Open details for ${row.label}` },
      });
      open.addEventListener("click", () => {
        // The detail panel is the single entity_detail fetch driver; selection
        // only moves focus through the state machine (I4).
        store.dispatch({ kind: "entityFocused", entityId: row.id });
      });
      body.append(
        el("tr", {}, [
          el("th", { attrs: { scope: "row" } }, [open]),
          el("td", { text: row.kindLabel }),
          el("td", { text: String(row.degree) }),
        ]),
      );
    }
    return el("table", {}, [
      el("caption", { className: "cn-ui-visually-hidden", text: "Entities in the current view" }),
      head,
      body,
    ]);
  }

  function render(): void {
    const state = store.getState();
    const projection = state.data.projection;
    const kindMeta = state.data.kindMeta;
    if (projection === renderedProjection && kindMeta === renderedKindMeta) {
      return;
    }
    renderedProjection = projection;
    renderedKindMeta = kindMeta;
    const rows = flatRows(projection, kindMeta);
    count.textContent = rows.length === 1 ? "1 entity" : `${rows.length} entities`;
    tableWrap.replaceChildren(buildTable(rows));
  }

  const unsubscribe = store.subscribe(render);
  render();

  return () => {
    unsubscribe();
    container.replaceChildren();
  };
}
