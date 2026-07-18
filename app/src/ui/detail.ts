import type { Store } from "../state/store";
import type { AppState, EntityDetailDto } from "../state/state";
import type { WasmClient } from "../wasm/client";
import { openDetail } from "../state/effects";
import { selectCurrentDetail, selectProjectedEntityById } from "../state/selectors";
import { entityLabel } from "../viz/projection";
import { el, uiId } from "./dom";
import {
  DRAFT_REVIEW_MARKER,
  TIER_CODES,
  TIER_PLAIN,
  detailAttributeViews,
  extractProvenance,
  isTierCode,
} from "./format";
import "./ui.css";

/**
 * P1.5 entity detail panel over cn-api `entity_detail`. TSDF tier codes are
 * the primary tier language with generic plain-language secondaries (D-032).
 * Provenance renders exactly what the payload carries - a one-line summary,
 * plus the full custody chain only when the projection includes one (D-033);
 * the panel makes no viewer-role decisions (I2).
 *
 * The panel is the single `entity_detail` fetch driver: any selection path
 * (3D picking, search, flat list) only dispatches `entityFocused`; when the
 * focused entity has no matching detail payload yet, the panel runs the
 * openDetail effect exactly once per focused entity (I4).
 */

export type DetailDeps = {
  readonly store: Store;
  readonly client: WasmClient;
};

export function mountDetailPanel(container: HTMLElement, deps: DetailDeps): () => void {
  const { store, client } = deps;
  const headingId = uiId("cn-detail-heading");
  const root = el("section", {
    className: "cn-detail",
    attrs: { "aria-labelledby": headingId, tabindex: "-1" },
  });
  root.hidden = true;
  container.replaceChildren(root);

  let renderedDetail: EntityDetailDto | null = null;
  let renderedEntityId: string | null = null;
  // Ephemeral fetch guard: which entity id the panel has already requested.
  let requestedEntityId: string | null = null;

  function close(): void {
    store.dispatch({ kind: "detailCleared" });
    store.dispatch({ kind: "entityFocusCleared" });
  }

  function headerRow(title: string, ownRecord: boolean): HTMLElement {
    const heading = el("h2", {
      className: "cn-detail-title",
      text: title,
      attrs: { id: headingId },
    });
    if (ownRecord) {
      heading.append(el("span", { className: "cn-detail-own", text: "Your record" }));
    }
    const closeButton = el("button", {
      className: "cn-detail-close",
      text: "Close",
      attrs: { type: "button", "aria-label": "Close detail panel" },
    });
    closeButton.addEventListener("click", close);
    return el("div", { className: "cn-detail-header" }, [heading, closeButton]);
  }

  function attributeList(detail: EntityDetailDto): HTMLElement | null {
    const views = detailAttributeViews(detail);
    if (views.length === 0) {
      return null;
    }
    const list = el("dl", { className: "cn-detail-attrs" });
    for (const view of views) {
      list.append(el("dt", { text: view.label }));
      const value = el("dd", { text: view.valueText });
      if (view.tier !== null) {
        value.append(
          el("span", {
            className: "cn-chip",
            text: view.tier,
            attrs: { title: `${TIER_PLAIN[view.tier]} - ${DRAFT_REVIEW_MARKER}` },
          }),
        );
      }
      if (view.visibility !== null) {
        value.append(el("span", { className: "cn-chip", text: view.visibility }));
      }
      list.append(value);
    }
    return list;
  }

  function provenanceSection(detail: EntityDetailDto): HTMLElement | null {
    const result = extractProvenance(detail);
    if (result.view === null && result.finding === null) {
      return null;
    }
    const section = el("div", { className: "cn-detail-provenance" });
    if (result.view !== null) {
      section.append(el("p", { text: result.view.line }));
    }
    if (result.view !== null && result.view.chain.length > 0) {
      section.append(
        el(
          "ul",
          { attrs: { "aria-label": "Custody chain" } },
          result.view.chain.map((event) => el("li", { text: event })),
        ),
      );
    }
    if (result.finding !== null) {
      section.append(
        el("p", {
          className: "cn-detail-finding",
          text: result.finding.message,
          attrs: { role: "alert" },
        }),
      );
    }
    return section;
  }

  function tierHelp(detail: EntityDetailDto): HTMLElement | null {
    if (!isTierCode(detail.tier) && !detailAttributeViews(detail).some((view) => view.tier !== null)) {
      return null;
    }
    const definitions = el("dl");
    for (const code of TIER_CODES) {
      definitions.append(el("dt", { text: code }), el("dd", { text: TIER_PLAIN[code] }));
    }
    return el("details", { className: "cn-tier-help" }, [
      el("summary", { text: `What the tier codes mean - ${DRAFT_REVIEW_MARKER}` }),
      definitions,
    ]);
  }

  function titleFor(state: AppState, entityId: string): string {
    const entity = selectProjectedEntityById(state, entityId);
    return entity === null ? entityId : entityLabel(entity, state.data.kindMeta);
  }

  function render(): void {
    const state = store.getState();
    const entityId = state.ui.detailEntityId;
    const detail = selectCurrentDetail(state);
    if (entityId === null) {
      requestedEntityId = null;
    } else if (detail === null && requestedEntityId !== entityId) {
      requestedEntityId = entityId;
      // Deferred so the effect's dispatches never re-enter the notification
      // loop; failures surface through errorSurfaced inside the effect (I3).
      queueMicrotask(() => {
        void openDetail(store, client, entityId);
      });
    }
    if (detail === renderedDetail && entityId === renderedEntityId) {
      return;
    }
    renderedDetail = detail;
    renderedEntityId = entityId;
    if (entityId === null) {
      root.hidden = true;
      root.replaceChildren();
      return;
    }
    root.hidden = false;
    if (detail === null) {
      root.replaceChildren(
        headerRow(titleFor(state, entityId), false),
        el("p", { className: "cn-detail-loading", text: "Loading", attrs: { "aria-live": "polite" } }),
      );
      return;
    }
    const kind = typeof detail.kind === "string" ? detail.kind : "";
    const kindLabel = state.data.kindMeta[kind]?.label ?? kind;
    const sections: HTMLElement[] = [
      headerRow(titleFor(state, entityId), detail.owner_is_viewer === true),
    ];
    if (kindLabel !== "") {
      sections.push(el("p", { className: "cn-detail-kind", text: kindLabel }));
    }
    if (isTierCode(detail.tier)) {
      sections.push(
        el("p", { className: "cn-detail-tier" }, [
          "Entity tier ",
          el("span", {
            className: "cn-chip",
            text: detail.tier,
            attrs: { title: `${TIER_PLAIN[detail.tier]} - ${DRAFT_REVIEW_MARKER}` },
          }),
        ]),
      );
    }
    for (const section of [attributeList(detail), provenanceSection(detail), tierHelp(detail)]) {
      if (section !== null) {
        sections.push(section);
      }
    }
    root.replaceChildren(...sections);
  }

  const unsubscribe = store.subscribe(render);
  render();

  return () => {
    unsubscribe();
    container.replaceChildren();
  };
}
