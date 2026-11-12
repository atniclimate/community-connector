import type { Store } from "../state/store";
import type {
  AppState,
  ContinuanceDto,
  IntroductionDto,
  JsonValue,
  KindMeta,
  PathDto,
  ProjectionEntityDto,
  RecordMirrorDto,
  RequestSlice,
  VisibilityMirrorDto,
} from "../state/state";
import type { WasmClient } from "../wasm/client";
import { loadContinuance, loadIntroductions, loadMirror, loadNeighborhood, ROUTE_MAX_HOPS, runRouting } from "../state/effects";
import {
  selectEgoNeighborhood,
  selectMirrorView,
  selectProjectedEntities,
  selectRoutingResults,
  selectSelfEntityId,
  type RoutingCandidateView,
} from "../state/selectors";
import { entityLabel } from "../viz/projection";
import { el, uiId } from "./dom";
import { formatAttributeValue, humanizeAttrId } from "./format";
import "./ui.css";

/**
 * Relatives panel: who can help, introductions, what rests on you, and the
 * record mirror. It renders only what the core already scoped to the viewer
 * (I2) and mutates state only by dispatching actions (I4). The pure view
 * model below is tested in relatives.test.ts; the DOM wiring stays thin.
 *
 * Every user-visible string here is DRAFT COPY in plain developer language,
 * pending ATNI vocabulary and community-language review (D-051, D-023).
 */

export const UNNAMED_RELATIVE = "a relative in this group";
export const NO_OWN_RECORD = "You have no record of your own in this group, so there is nothing person-centered to show yet.";
export const INTRODUCTION_LIMIT = 5;
export const AUDIENCES = [
  ["public", "Public"],
  ["network", "Network"],
  ["group", "Group"],
  ["trusted", "Trusted"],
] as const;

export type RelativesContext = {
  readonly selfId: string | null;
  readonly entities: ReadonlyMap<string, ProjectionEntityDto>;
  readonly kindMeta: Readonly<Record<string, KindMeta>>;
};

export function relativesContext(state: AppState): RelativesContext {
  return {
    selfId: selectSelfEntityId(state),
    entities: new Map(selectProjectedEntities(state).map((entity) => [entity.id, entity])),
    kindMeta: state.data.kindMeta,
  };
}

function isRecord(value: unknown): value is Readonly<Record<string, JsonValue>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function attribute(entity: ProjectionEntityDto | undefined, id: string): JsonValue | undefined {
  const attributes = entity?.attributes;
  return isRecord(attributes) ? attributes[id] : undefined;
}

export function countText(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

export function labelOf(ctx: RelativesContext, id: string): string {
  const entity = ctx.entities.get(id);
  return entity === undefined ? UNNAMED_RELATIVE : entityLabel(entity, ctx.kindMeta);
}

/**
 * True only when the viewer's projection shows willing_to_introduce = yes.
 * A hidden value reads exactly like "no" (ADR-003 A-B4); this reads what the
 * core already filtered and decides no visibility (I2).
 */
export function willingToIntroduce(entity: ProjectionEntityDto | undefined): boolean {
  const value = attribute(entity, "willing_to_introduce");
  return isRecord(value) && value.value === "yes";
}

function introducerName(ctx: RelativesContext, id: string): string {
  return willingToIntroduce(ctx.entities.get(id)) ? `${labelOf(ctx, id)} (willing to introduce)` : UNNAMED_RELATIVE;
}

/** The chain of relatives from the viewer to a result, naming only willing introducers. */
export function chainText(ctx: RelativesContext, path: PathDto | null): string {
  if (path === null) {
    return `No chain of relatives within ${ROUTE_MAX_HOPS} steps reaches them.`;
  }
  const middle = path.nodes.slice(1, -1);
  if (middle.length === 0) {
    return "You are already directly connected.";
  }
  return `From you ${middle.map((id) => `through ${introducerName(ctx, id)}`).join(", then ")}.`;
}

export type HelperView = {
  readonly entity: string;
  readonly label: string;
  readonly offering: string;
  readonly chain: string;
  readonly resting: boolean;
};

export function helperViews(ctx: RelativesContext, candidates: readonly RoutingCandidateView[]): readonly HelperView[] {
  return candidates.map((candidate) => ({
    entity: candidate.entity,
    label: labelOf(ctx, candidate.entity),
    offering: formatAttributeValue(attribute(ctx.entities.get(candidate.entity), "offering")),
    chain: chainText(ctx, candidate.path),
    resting: candidate.resting,
  }));
}

export type IntroductionView = {
  readonly entity: string;
  readonly label: string;
  readonly reason: string;
  readonly shared: readonly string[];
};

function introducerList(ctx: RelativesContext, introducers: IntroductionDto["introducers"]): string {
  const named = introducers.filter((i) => willingToIntroduce(ctx.entities.get(i.entity))).map((i) => introducerName(ctx, i.entity));
  const unnamed = introducers.length - named.length;
  const rest = unnamed === 0 ? [] : [unnamed === 1 ? UNNAMED_RELATIVE : `${unnamed} relatives in this group`];
  return [...named, ...rest].join(" and ");
}

/** The top suggestions in core order, with the same introducer naming rule as the chain. */
export function introductionViews(ctx: RelativesContext, suggestions: readonly IntroductionDto[]): readonly IntroductionView[] {
  return suggestions.slice(0, INTRODUCTION_LIMIT).map((suggestion) => {
    const { center_seeks: theyOffer, candidate_seeks: theySeek } = suggestion.shared_tags;
    const shared = [
      ...(theyOffer.length > 0 ? [`They offer what you seek: ${theyOffer.join(", ")}`] : []),
      ...(theySeek.length > 0 ? [`They seek what you offer: ${theySeek.join(", ")}`] : []),
    ];
    const relatives = countText(suggestion.introducers.length, "mutual relative", "mutual relatives");
    return {
      entity: suggestion.candidate,
      label: labelOf(ctx, suggestion.candidate),
      reason: `You share ${relatives}: ${introducerList(ctx, suggestion.introducers)}.`,
      shared,
    };
  });
}

function isOwn(ctx: RelativesContext, id: string): boolean {
  return ctx.entities.get(id)?.owner_is_viewer === true;
}

/**
 * Plain sentences for what rests on the viewer, in core order (no ranking).
 * The core already scoped continuance to the viewer's own records.
 */
export function restingOnYouLines(ctx: RelativesContext, continuance: ContinuanceDto): readonly string[] {
  const subject = (id: string): string => (id === ctx.selfId ? "You are" : `${labelOf(ctx, id)} is`);
  const possessive = (id: string): string => (id === ctx.selfId ? "Your" : `${labelOf(ctx, id)}'s`);
  const points = Object.values(continuance.articulation_points).map(
    (point) =>
      `${subject(point.entity)} the only tie between ${point.parts} parts of this group, of ${point.part_sizes.join(", ")} people.`,
  );
  const bridges = Object.values(continuance.bridges).map((tie) => {
    const fromIsOwn = isOwn(ctx, tie.from);
    const [own, other] = fromIsOwn ? [tie.from, tie.to] : [tie.to, tie.from];
    const [ownSide, otherSide] = fromIsOwn ? [tie.from_side, tie.to_side] : [tie.to_side, tie.from_side];
    return `${possessive(own)} tie with ${labelOf(ctx, other)} is the only link between a group of ${ownSide} and a group of ${otherSide}.`;
  });
  return [...points, ...bridges];
}

export type MirrorRow = {
  readonly label: string;
  /** One flag per AUDIENCES entry, in that order. */
  readonly receives: readonly boolean[];
};

/** One row per value of the viewer's own records: presence first, then each attribute. */
export function mirrorRows(ctx: RelativesContext, mirror: RecordMirrorDto): readonly MirrorRow[] {
  const entries = Object.entries(mirror.entities);
  const rows: MirrorRow[] = [];
  for (const [entityId, entity] of entries) {
    const prefix = entries.length > 1 ? `${labelOf(ctx, entityId)}: ` : "";
    const values: [string, VisibilityMirrorDto][] = [
      ["the record itself", entity.presence],
      ...Object.entries(entity.attributes).map(([attr, value]): [string, VisibilityMirrorDto] => [humanizeAttrId(attr), value]),
    ];
    for (const [label, value] of values) {
      rows.push({
        label: `${prefix}${label}`,
        receives: AUDIENCES.map(([audience]) => value.audiences[audience] === true),
      });
    }
  }
  return rows;
}

/** A loading or error line for a slice, or null when it is ready or idle. */
export function sliceNotice(slice: RequestSlice<unknown>, what: string): string | null {
  if (slice.status === "pending") {
    return `Loading ${what}.`;
  }
  return slice.status === "error" ? `Could not load ${what}: ${slice.error?.message ?? "unknown error"}` : null;
}

export function routingStatusText(state: AppState): string {
  const { status, result, error } = state.routing;
  if (status === "pending") {
    return "Searching";
  }
  if (status === "error") {
    return `Search failed: ${error?.message ?? "unknown error"}`;
  }
  return status === "ready" ? countText(result?.length ?? 0, "relative can help", "relatives can help") : "";
}

export const LAYOUT_NO_RECORD = "My relatives needs a record of your own in this group.";
export const LAYOUT_WAITING = "My relatives is waiting for your relatives to load.";

export type LayoutToggleView = {
  /** True when the ring layout can be shown. */
  readonly available: boolean;
  /** True when the ring layout is the one on screen. */
  readonly mine: boolean;
  /** Why My relatives is disabled, or "" when it is available. */
  readonly note: string;
};

export function layoutToggleView(state: AppState): LayoutToggleView {
  const available = selectEgoNeighborhood(state) !== null;
  const note = available
    ? ""
    : selectSelfEntityId(state) === null
      ? LAYOUT_NO_RECORD
      : sliceNotice(state.analysis.neighborhood, "your relatives") ?? LAYOUT_WAITING;
  return { available, mine: available && state.ui.layoutMode === "mine", note };
}

// ---- DOM wiring (thin) ----

/** Whole network / My relatives switch; updated in place so focus survives renders (I9). */
function layoutToggle(store: Store): { readonly element: HTMLElement; readonly update: (view: LayoutToggleView) => void } {
  const noteId = uiId("cn-relatives-layout-note");
  const button = (text: string, mode: "whole" | "mine"): HTMLButtonElement => {
    const b = el("button", { className: "cn-relatives-submit", text, attrs: { type: "button", "aria-pressed": "false" } });
    b.addEventListener("click", () => store.dispatch({ kind: "layoutModeChanged", mode }));
    return b;
  };
  const whole = button("Whole network", "whole");
  const mine = button("My relatives", "mine");
  const note = el("p", { className: "cn-relatives-line", attrs: { id: noteId } });
  const element = el("div", { className: "cn-relatives-form", attrs: { role: "group", "aria-label": "Graph layout" } }, [whole, mine, note]);
  return {
    element,
    update: (view) => {
      whole.setAttribute("aria-pressed", String(!view.mine));
      mine.setAttribute("aria-pressed", String(view.mine));
      mine.disabled = !view.available;
      note.textContent = view.note;
      note.hidden = view.note === "";
      if (view.note === "") {
        mine.removeAttribute("aria-describedby");
      } else {
        mine.setAttribute("aria-describedby", noteId);
      }
    },
  };
}

export type RelativesDeps = {
  readonly store: Store;
  readonly client: WasmClient;
};

function region(title: string, children: readonly HTMLElement[]): HTMLElement {
  const id = uiId("cn-relatives-heading");
  return el("section", { className: "cn-relatives-section", attrs: { role: "region", "aria-labelledby": id } }, [
    el("h3", { className: "cn-relatives-heading", text: title, attrs: { id } }),
    ...children,
  ]);
}

function lines(texts: readonly string[], className = "cn-relatives-line"): HTMLElement[] {
  return texts.map((text) => el("p", { className, text }));
}

function pick(label: string, onPick: () => void): HTMLButtonElement {
  const button = el("button", { className: "cn-relatives-pick", text: label, attrs: { type: "button" } });
  button.addEventListener("click", onPick);
  return button;
}

function helperList(store: Store, views: readonly HelperView[]): HTMLElement {
  return el(
    "ul",
    { className: "cn-relatives-list" },
    views.map((view) =>
      el("li", {}, [
        pick(view.label, () => store.dispatch({ kind: "routeResultChosen", entityId: view.entity })),
        ...lines([
          view.offering === "" ? "No offering tags shown." : `Offers: ${view.offering}`,
          view.chain,
          ...(view.resting ? ["Resting for now."] : []),
        ]),
      ]),
    ),
  );
}

function introductionList(store: Store, views: readonly IntroductionView[]): HTMLElement {
  return el(
    "ul",
    { className: "cn-relatives-list" },
    views.map((view) =>
      el("li", {}, [
        pick(view.label, () => store.dispatch({ kind: "entityFocused", entityId: view.entity })),
        ...lines([view.reason, ...view.shared]),
      ]),
    ),
  );
}

function mirrorTable(rows: readonly MirrorRow[]): HTMLElement {
  const head = el("tr", {}, [
    el("th", { text: "Value", attrs: { scope: "col" } }),
    ...AUDIENCES.map(([, title]) => el("th", { text: title, attrs: { scope: "col" } })),
  ]);
  const body = rows.map((row) =>
    el("tr", {}, [
      el("th", { text: row.label, attrs: { scope: "row" } }),
      ...row.receives.map((receives) => el("td", { text: receives ? "Receives" : "Does not" })),
    ]),
  );
  return el("div", { className: "cn-flat-scroll" }, [
    el("table", {}, [
      el("caption", { className: "cn-ui-visually-hidden", text: "What each audience receives of your records" }),
      el("thead", {}, [head]),
      el("tbody", {}, body),
    ]),
  ]);
}

function sliceBody(slice: RequestSlice<unknown>, what: string, ready: () => HTMLElement[]): HTMLElement[] {
  const notice = sliceNotice(slice, what);
  return notice !== null ? lines([notice]) : slice.status === "ready" ? ready() : [];
}

function analysisSections(store: Store, state: AppState, ctx: RelativesContext): HTMLElement[] {
  const { introductions, continuance, mirror } = state.analysis;
  const mirrorView = selectMirrorView(state);
  return [
    region("Introductions you could ask for", sliceBody(introductions, "introductions", () => {
      const views = introductionViews(ctx, introductions.result ?? []);
      return views.length === 0 ? lines(["No introductions to suggest right now."]) : [introductionList(store, views)];
    })),
    region("What rests on you", sliceBody(continuance, "what rests on you", () => {
      const found = continuance.result === null ? [] : restingOnYouLines(ctx, continuance.result);
      return found.length === 0
        ? lines(["Nothing in this group rests on you alone right now."])
        : lines(["These ties rest on you. You could share the load by introducing the people on each side to one another.", ...found]);
    })),
    region("See your record as", sliceBody(mirror, "your record mirror", () => [
      mirrorTable(mirror.result === null ? [] : mirrorRows(ctx, mirror.result)),
      ...lines(mirrorView.gaps.map((gap) => gap.text), "cn-relatives-gap"),
    ])),
  ];
}

const LOADERS = [
  ["continuance", loadContinuance],
  ["introductions", loadIntroductions],
  ["mirror", loadMirror],
  ["neighborhood", loadNeighborhood],
] as const;

export function mountRelatives(container: HTMLElement, deps: RelativesDeps): () => void {
  const { store, client } = deps;
  const inputId = uiId("cn-relatives-input");
  const headingId = uiId("cn-relatives-title");
  const input = el("input", { className: "cn-search-input", attrs: { id: inputId, type: "text", autocomplete: "off" } });
  const form = el("form", { className: "cn-relatives-form" }, [
    el("label", { className: "cn-search-label", text: "Describe what you need", attrs: { for: inputId } }),
    input,
    el("button", { className: "cn-relatives-submit", text: "Find relatives", attrs: { type: "submit" } }),
  ]);
  const helpers = el("div");
  const rest = el("div");
  const body = el("div", {}, [region("Who can help with...", [form, helpers]), rest]);
  const none = el("p", { className: "cn-relatives-line", text: NO_OWN_RECORD });
  const live = el("div", { className: "cn-ui-visually-hidden", attrs: { role: "status", "aria-live": "polite" } });
  const toggle = layoutToggle(store);
  container.replaceChildren(
    el("section", { className: "cn-relatives", attrs: { "aria-labelledby": headingId } }, [
      el("h2", { className: "cn-flat-title", text: "Your relatives", attrs: { id: headingId } }),
      toggle.element,
      none,
      body,
      live,
    ]),
  );

  let rendered: readonly unknown[] = [];
  let tornDown = false;

  function ensureLoaded(): void {
    const state = store.getState();
    if (tornDown || selectSelfEntityId(state) === null) {
      return;
    }
    for (const [name, load] of LOADERS) {
      if (state.analysis[name].status === "idle") {
        void load(store, client);
      }
    }
  }

  function render(): void {
    const state = store.getState();
    const { routing } = state;
    // routing.chosen is left out: choosing a result must not rebuild the list
    // and drop keyboard focus from the button just pressed (I9).
    const keys = [state.data.projection, state.data.kindMeta, routing.status, routing.result, routing.error, state.analysis, state.ui.layoutMode];
    if (keys.every((key, index) => key === rendered[index])) {
      return;
    }
    rendered = keys;
    toggle.update(layoutToggleView(state));
    const ctx = relativesContext(state);
    none.hidden = ctx.selfId !== null || state.data.projection === null;
    body.hidden = ctx.selfId === null;
    const views = helperViews(ctx, selectRoutingResults(state));
    helpers.replaceChildren(
      ...lines([routingStatusText(state)].filter((text) => text !== "")),
      ...(views.length > 0 ? [helperList(store, views)] : []),
    );
    rest.replaceChildren(...analysisSections(store, state, ctx));
    const introCount = state.analysis.introductions.result?.length;
    live.textContent = [
      routingStatusText(state),
      introCount === undefined ? "" : countText(Math.min(introCount, INTRODUCTION_LIMIT), "introduction suggested", "introductions suggested"),
    ].filter((t) => t !== "").join(". ");
    // Deferred so a load's own dispatch never runs inside this listener.
    queueMicrotask(ensureLoaded);
  }

  const onSubmit = (event: SubmitEvent): void => {
    event.preventDefault();
    void runRouting(store, client, input.value);
  };
  form.addEventListener("submit", onSubmit);
  const unsubscribe = store.subscribe(render);
  render();

  return () => {
    tornDown = true;
    form.removeEventListener("submit", onSubmit);
    unsubscribe();
    container.replaceChildren();
  };
}
