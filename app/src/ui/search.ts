import type { Store } from "../state/store";
import type { SearchState } from "../state/state";
import type { WasmClient } from "../wasm/client";
import { runSearch } from "../state/effects";
import { selectProjectedEntityById, selectSearch } from "../state/selectors";
import { entityLabel } from "../viz/projection";
import { el, uiId } from "./dom";
import "./ui.css";

/**
 * P1.4 search box over the existing cn-api `search` operation (attribute hits
 * only). Debounced input; combobox/listbox keyboard pattern (I9); every data
 * mutation flows through the state machine (I4) - the only component-local
 * value is the ephemeral listbox cursor.
 */

export const SEARCH_DEBOUNCE_MS = 250;
const NO_ACTIVE_OPTION = -1;

export type SearchDeps = {
  readonly store: Store;
  readonly client: WasmClient;
};

export function mountSearch(container: HTMLElement, deps: SearchDeps): () => void {
  const { store, client } = deps;
  const inputId = uiId("cn-search-input");
  const listboxId = uiId("cn-search-listbox");

  const label = el("label", {
    className: "cn-search-label",
    text: "Search",
    attrs: { for: inputId },
  });
  const input = el("input", {
    className: "cn-search-input",
    attrs: {
      id: inputId,
      type: "text",
      role: "combobox",
      "aria-expanded": "false",
      "aria-controls": listboxId,
      "aria-autocomplete": "list",
      "aria-label": "Search entities",
      autocomplete: "off",
      spellcheck: "false",
    },
  });
  const status = el("div", {
    className: "cn-search-status",
    attrs: { role: "status", "aria-live": "polite" },
  });
  const listbox = el("ul", {
    className: "cn-search-results",
    attrs: { id: listboxId, role: "listbox", "aria-label": "Search results" },
  });
  listbox.hidden = true;
  const root = el("div", { className: "cn-search" }, [label, input, status, listbox]);
  container.replaceChildren(root);

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let activeIndex = NO_ACTIVE_OPTION;
  let renderedSearch: SearchState | null = null;

  function scheduleSearch(): void {
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      debounceTimer = null;
      void runSearch(store, client, input.value);
    }, SEARCH_DEBOUNCE_MS);
  }

  function select(index: number): void {
    const hit = store.getState().search.hits[index];
    if (hit === undefined) {
      return;
    }
    // The detail panel is the single entity_detail fetch driver; selection
    // only moves focus through the state machine (I4).
    store.dispatch({ kind: "entityFocused", entityId: hit.entity });
  }

  function moveActive(delta: number): void {
    const count = store.getState().search.hits.length;
    if (count === 0) {
      return;
    }
    activeIndex = (activeIndex + delta + count) % count;
    renderOptions(store.getState().search);
  }

  function onKeydown(event: KeyboardEvent): void {
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        moveActive(1);
        break;
      case "ArrowUp":
        event.preventDefault();
        moveActive(-1);
        break;
      case "Enter":
        if (activeIndex !== NO_ACTIVE_OPTION) {
          event.preventDefault();
          select(activeIndex);
        }
        break;
      case "Escape":
        event.preventDefault();
        input.value = "";
        store.dispatch({ kind: "searchCleared" });
        break;
      default:
        break;
    }
  }

  function optionPrimaryText(entityId: string, snippet: string): string {
    const state = store.getState();
    const entity = selectProjectedEntityById(state, entityId);
    return entity === null ? snippet : entityLabel(entity, state.data.kindMeta);
  }

  function renderOptions(search: SearchState): void {
    const state = store.getState();
    if (activeIndex >= search.hits.length) {
      activeIndex = NO_ACTIVE_OPTION;
    }
    listbox.replaceChildren(
      ...search.hits.map((hit, index) => {
        const kindLabel = state.data.kindMeta[hit.kind]?.label ?? hit.kind;
        const option = el(
          "li",
          {
            className: "cn-search-option",
            attrs: {
              id: `${listboxId}-opt-${index}`,
              role: "option",
              "aria-selected": index === activeIndex ? "true" : "false",
            },
          },
          [
            optionPrimaryText(hit.entity, hit.snippet),
            el("span", {
              className: "cn-search-option-secondary",
              text: `${kindLabel} - ${hit.matched_attr}: ${hit.snippet}`,
            }),
          ],
        );
        option.addEventListener("click", () => {
          select(index);
        });
        return option;
      }),
    );
    const expanded = search.hits.length > 0;
    listbox.hidden = !expanded;
    input.setAttribute("aria-expanded", expanded ? "true" : "false");
    if (activeIndex === NO_ACTIVE_OPTION) {
      input.removeAttribute("aria-activedescendant");
    } else {
      input.setAttribute("aria-activedescendant", `${listboxId}-opt-${activeIndex}`);
    }
  }

  function statusText(search: SearchState): string {
    switch (search.status) {
      case "pending":
        return "Searching";
      case "ready":
        return search.hits.length === 1 ? "1 result" : `${search.hits.length} results`;
      case "error":
        return `Search failed: ${search.error?.message ?? "unknown error"}`;
      case "idle":
        return "";
      default:
        return "";
    }
  }

  function render(): void {
    const search = selectSearch(store.getState());
    if (search === renderedSearch) {
      return;
    }
    if (renderedSearch === null || search.hits !== renderedSearch.hits) {
      activeIndex = NO_ACTIVE_OPTION;
    }
    renderedSearch = search;
    status.textContent = statusText(search);
    renderOptions(search);
  }

  input.addEventListener("input", scheduleSearch);
  input.addEventListener("keydown", onKeydown);
  const unsubscribe = store.subscribe(render);
  render();

  return () => {
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
    }
    unsubscribe();
    container.replaceChildren();
  };
}
