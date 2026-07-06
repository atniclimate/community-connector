import { Raycaster, Vector2, type Camera, type Intersection } from "three";
import type { Store } from "../state/store";
import type { NodeLayer } from "./nodes";

export type PickDispatch = Pick<Store, "dispatch">;
export type RaycastFn = (
  raycaster: Raycaster,
  layer: NodeLayer,
) => readonly Intersection[];

const FIRST_HIT = 0;
const NO_INSTANCE = -1;
const DOUBLE = 2;
type ClientPoint = Pick<MouseEvent, "clientX" | "clientY">;

export function entityIdFromIntersection(
  intersection: Intersection | undefined,
  layer: NodeLayer,
): string | null {
  if (intersection === undefined || intersection.instanceId === undefined) {
    return null;
  }
  const ids = layer.meshToEntityIds.get(intersection.object as NodeLayer["records"][number]["mesh"]);
  return ids?.[intersection.instanceId] ?? null;
}

export function dispatchHoveredEntity(store: PickDispatch, entityId: string | null): void {
  store.dispatch({ kind: "entityHovered", entityId });
}

export function dispatchPickedEntity(store: PickDispatch, entityId: string): void {
  store.dispatch({ kind: "entityFocused", entityId });
}

function normalizedPointer(event: ClientPoint, element: HTMLElement): Vector2 {
  const rect = element.getBoundingClientRect();
  return new Vector2(
    ((event.clientX - rect.left) / rect.width) * DOUBLE - 1,
    -(((event.clientY - rect.top) / rect.height) * DOUBLE - 1),
  );
}

function defaultRaycast(raycaster: Raycaster, layer: NodeLayer): readonly Intersection[] {
  return raycaster.intersectObjects(layer.records.map((record) => record.mesh), false);
}

export class PickingController {
  private readonly raycaster = new Raycaster();
  private readonly raycast: RaycastFn;
  private pending: PointerEvent | null = null;
  private frame = NO_INSTANCE;

  public constructor(
    private readonly element: HTMLElement,
    private readonly camera: Camera,
    private readonly layerProvider: () => NodeLayer | null,
    private readonly store: PickDispatch,
    raycast: RaycastFn = defaultRaycast,
  ) {
    this.raycast = raycast;
    this.element.addEventListener("pointermove", this.onPointerMove);
    this.element.addEventListener("pointerleave", this.onPointerLeave);
    this.element.addEventListener("click", this.onClick);
  }

  public dispose(): void {
    this.element.removeEventListener("pointermove", this.onPointerMove);
    this.element.removeEventListener("pointerleave", this.onPointerLeave);
    this.element.removeEventListener("click", this.onClick);
    if (this.frame !== NO_INSTANCE) {
      cancelAnimationFrame(this.frame);
    }
  }

  public pickNow(event: ClientPoint): string | null {
    const layer = this.layerProvider();
    if (layer === null) {
      return null;
    }
    this.raycaster.setFromCamera(normalizedPointer(event, this.element), this.camera);
    return entityIdFromIntersection(this.raycast(this.raycaster, layer)[FIRST_HIT], layer);
  }

  private readonly onPointerMove = (event: PointerEvent): void => {
    this.pending = event;
    if (this.frame !== NO_INSTANCE) {
      return;
    }
    this.frame = requestAnimationFrame(() => {
      this.frame = NO_INSTANCE;
      const pending = this.pending;
      this.pending = null;
      dispatchHoveredEntity(this.store, pending === null ? null : this.pickNow(pending));
    });
  };

  private readonly onPointerLeave = (): void => {
    this.pending = null;
    dispatchHoveredEntity(this.store, null);
  };

  private readonly onClick = (event: MouseEvent): void => {
    const entityId = this.pickNow(event);
    if (entityId !== null) {
      dispatchPickedEntity(this.store, entityId);
    }
  };
}
