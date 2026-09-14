import type { AppState, ProjectionEntityDto, ViewMode } from "../state/state";
import { entityLabel } from "./projection";

const TOOLTIP_OFFSET = 14;
const TOOLTIP_MARGIN = 8;
const ZERO = 0;

/**
 * The stage name gate for hover (D-099: no person name ever appears on
 * stage). In present mode the tooltip is suppressed entirely rather than
 * reduced to the kind label: it is the simpler rule, the operator does not
 * need a tooltip on stage, and the node's hover emphasis still shows which
 * node the pointer is on. Pure so it is testable without a DOM.
 */
export function stageTooltipSuppressed(mode: ViewMode): boolean {
  return mode === "present";
}

/** Writes (or clears) one node's hover emphasis on the current node layer. */
export type HoverWriter = (entityId: string, hovered: boolean) => void;

/**
 * Projects `view.hoveredEntityId` onto the graph: node emphasis through the
 * supplied writer plus one pointer-anchored tooltip. It never mutates app
 * state (I4); hover changes arrive only through `entityHovered`.
 *
 * The tooltip is a visual echo for pointer users and is aria-hidden: the flat
 * view and search are the keyboard and screen-reader paths to the same names
 * (D-098 records this exception to the brief's keyboard-tooltip rule).
 */
export class HoverOverlay {
  private readonly tooltip: HTMLDivElement;
  private readonly pointer = { x: ZERO, y: ZERO };
  private current: string | null = null;

  public constructor(
    private readonly container: HTMLElement,
    private readonly canvas: HTMLElement,
  ) {
    this.tooltip = document.createElement("div");
    this.tooltip.className = "cn-viz-tooltip";
    this.tooltip.setAttribute("aria-hidden", "true");
    this.tooltip.hidden = true;
    container.append(this.tooltip);
    canvas.addEventListener("pointermove", this.onPointerMove);
  }

  public get hoveredEntityId(): string | null {
    return this.current;
  }

  /**
   * `rebuilt` forces a refresh even when the id is unchanged: the entity's
   * label may have changed, or it may no longer be projected at all.
   */
  public sync(
    state: AppState,
    entities: ReadonlyMap<string, ProjectionEntityDto>,
    rebuilt: boolean,
    write: HoverWriter,
  ): void {
    const next = state.view.hoveredEntityId;
    if (next === this.current && !rebuilt) {
      return;
    }
    if (next !== this.current) {
      if (this.current !== null) {
        write(this.current, false);
      }
      this.current = next;
      if (next !== null) {
        write(next, true);
      }
    }
    const entity = next === null || stageTooltipSuppressed(state.view.mode) ? undefined : entities.get(next);
    if (entity === undefined) {
      this.tooltip.hidden = true;
      return;
    }
    const name = document.createElement("strong");
    name.textContent = entityLabel(entity, state.data.kindMeta);
    const kind = document.createElement("span");
    kind.textContent = state.data.kindMeta[entity.kind ?? ""]?.label ?? entity.kind ?? "";
    this.tooltip.replaceChildren(name, kind);
    this.tooltip.hidden = false;
    this.position();
  }

  public dispose(): void {
    this.canvas.removeEventListener("pointermove", this.onPointerMove);
    this.tooltip.remove();
  }

  private readonly onPointerMove = (event: PointerEvent): void => {
    const rect = this.container.getBoundingClientRect();
    this.pointer.x = event.clientX - rect.left;
    this.pointer.y = event.clientY - rect.top;
    if (!this.tooltip.hidden) {
      this.position();
    }
  };

  private position(): void {
    const maxX = Math.max(ZERO, this.container.clientWidth - this.tooltip.offsetWidth - TOOLTIP_MARGIN);
    const maxY = Math.max(ZERO, this.container.clientHeight - this.tooltip.offsetHeight - TOOLTIP_MARGIN);
    const x = Math.max(TOOLTIP_MARGIN, Math.min(maxX, this.pointer.x + TOOLTIP_OFFSET));
    const y = Math.max(TOOLTIP_MARGIN, Math.min(maxY, this.pointer.y + TOOLTIP_OFFSET));
    this.tooltip.style.transform = `translate(${x}px, ${y}px)`;
  }
}
