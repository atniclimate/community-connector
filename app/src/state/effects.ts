import type { Store } from "./store";
import type { JsonObject, KindMeta, ShapeName, ViewerContextDto } from "./state";
import type { WasmClient } from "../wasm/client";
import { deriveTheme } from "../theme/derive";
import type { GroupTemplateDto, GroupTemplateKindDto } from "../theme/tokens";

const SHAPES: readonly ShapeName[] = ["sphere", "cube", "octahedron", "tetrahedron", "torus", "cone"];

function revisionFromProjection(projection: JsonObject): number {
  const revision = projection.revision;
  return typeof revision === "number" ? revision : 0;
}

function isShapeName(value: string | undefined): value is ShapeName {
  return SHAPES.includes(value as ShapeName);
}

function kindMetaFromTemplateKind(kind: GroupTemplateKindDto): KindMeta {
  return {
    shape: isShapeName(kind.shape) ? kind.shape : "sphere",
    label: kind.label ?? kind.id,
    colorRole: kind.color_role,
  };
}

function kindMetaFromTemplate(template: GroupTemplateDto): Readonly<Record<string, KindMeta>> {
  return Object.fromEntries(
    template.kinds.map((kind) => [kind.id, kindMetaFromTemplateKind(kind)]),
  );
}

/** Implements docs/blueprints/app-state.md effects and ADR-003 D5 worker dispatch. */
export async function loadGroup(
  store: Store,
  client: WasmClient,
  groupId: string,
  viewer: ViewerContextDto,
  templateJson: string,
  opsJsonl = "",
): Promise<void> {
  store.dispatch({ kind: "groupLoadRequested", groupId, viewer });
  try {
    await client.loadGroup(groupId, viewer, templateJson, opsJsonl);
    const template = JSON.parse(templateJson) as GroupTemplateDto;
    const derived = deriveTheme(template);
    store.dispatch({
      kind: "themeDerived",
      theme: derived.theme,
      report: derived.report,
      kindMeta: kindMetaFromTemplate(template),
    });
    store.dispatch({ kind: "groupLoadSucceeded", groupId });
    await refreshProjection(store, client);
  } catch (error) {
    store.dispatch({ kind: "groupLoadFailed", groupId, error: client.toErrorEnvelope(error) });
  }
}

/** Implements docs/blueprints/app-state.md promise-to-dispatch boundary. */
export async function refreshProjection(store: Store, client: WasmClient): Promise<void> {
  const state = store.getState();
  if (state.session.groupId === null) {
    return;
  }
  try {
    const projection = await client.projection(state.session.groupId, state.session.viewer);
    store.dispatch({
      kind: "projectionReceived",
      projection,
      revision: revisionFromProjection(projection),
    });
  } catch (error) {
    store.dispatch({ kind: "errorSurfaced", error: client.toErrorEnvelope(error) });
  }
}

/** Implements docs/blueprints/app-state.md stale detail tagging requirement. */
export async function openDetail(
  store: Store,
  client: WasmClient,
  entityId: string,
): Promise<void> {
  const state = store.getState();
  if (state.session.groupId === null) {
    return;
  }
  const revision = state.session.revision;
  store.dispatch({ kind: "entityFocused", entityId });
  try {
    const detail = await client.entityDetail(state.session.groupId, state.session.viewer, entityId);
    store.dispatch({ kind: "detailReceived", detail, entityId, revision });
  } catch (error) {
    store.dispatch({ kind: "errorSurfaced", error: client.toErrorEnvelope(error) });
  }
}
