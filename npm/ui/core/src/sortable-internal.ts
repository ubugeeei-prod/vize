import { toValue } from "vue";

import type {
  DragSourceRegistration,
  DropTargetRegistration,
} from "./drag-and-drop-controller-types.ts";
import type { DragPointerType, DropEdge } from "./drag-and-drop-types.ts";
import type {
  SortableAnnouncementContext,
  SortableAnnouncements,
  SortableDirection,
  SortableEvent,
  SortableEventType,
  SortableItemOptions,
  SortableOptions,
  SortableOrientation,
  SortablePosition,
} from "./sortable-types.ts";

/** Paired drag-and-drop registrations owned by one sortable item. */
export interface SortableItemRecord {
  readonly options: SortableItemOptions;
  readonly source: DragSourceRegistration;
  readonly target: DropTargetRegistration;
}

/** Mutable state for one active sort owned by a controller. */
export interface SortableSession {
  readonly key: string;
  readonly pointerType: DragPointerType;
  readonly originIndex: number;
  toIndex: number;
  overKey: string | null;
  position: SortablePosition | null;
}

const invalidOptionDiagnostic = "VIZE_UI_SORTABLE_OPTION";
const orientations = new Set<SortableOrientation>(["grid", "horizontal", "vertical"]);
const directions = new Set<SortableDirection>(["ltr", "rtl"]);

/** Resolve the closed orientation union without accepting mistyped JavaScript. */
export function readOrientation(value: SortableOptions["orientation"]): SortableOrientation {
  const resolved = toValue(value) ?? "vertical";
  if (!orientations.has(resolved)) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: orientation must resolve to grid, horizontal, or vertical`,
    );
  }
  return resolved;
}

/** Resolve the closed direction union without accepting mistyped JavaScript. */
export function readDirection(value: SortableOptions["direction"]): SortableDirection {
  const resolved = toValue(value) ?? "ltr";
  if (!directions.has(resolved)) {
    throw new TypeError(`${invalidOptionDiagnostic}: direction must resolve to ltr or rtl`);
  }
  return resolved;
}

/** Resolve a positive integer column count for grid arrow-key geometry. */
export function readColumns(value: SortableOptions["columns"]): number {
  const resolved = toValue(value) ?? 1;
  if (!Number.isInteger(resolved) || resolved < 1) {
    throw new TypeError(`${invalidOptionDiagnostic}: columns must resolve to an integer >= 1`);
  }
  return resolved;
}

/** Drop edges one item target resolves for an orientation. */
export function edgesFor(orientation: SortableOrientation, nesting: boolean): readonly DropEdge[] {
  const directional: DropEdge[] =
    orientation === "vertical" ? ["top", "bottom"] : ["left", "right"];
  return nesting ? [...directional, "inside"] : directional;
}

/** Map a logical position back onto the physical edge for an orientation. */
export function edgeForPosition(
  position: SortablePosition,
  orientation: SortableOrientation,
  direction: SortableDirection,
): DropEdge {
  if (position === "inside") return "inside";
  if (orientation === "vertical") return position === "before" ? "top" : "bottom";
  const before = direction === "rtl" ? "right" : "left";
  const after = direction === "rtl" ? "left" : "right";
  return position === "before" ? before : after;
}

/** Build a frozen lifecycle snapshot for sortable callbacks. */
export function createSortableEvent<Type extends SortableEventType>(
  type: Type,
  pointerType: DragPointerType,
  key: string,
  fromIndex: number,
  toIndex: number,
  overKey: string | null,
  position: SortablePosition | null,
  originalEvent: Event | null,
): SortableEvent<Type> {
  return Object.freeze({
    type,
    pointerType,
    key,
    fromIndex,
    toIndex,
    overKey,
    position,
    originalEvent,
  });
}

/** Map a physical drop edge onto the logical position for an orientation. */
export function positionForEdge(
  edge: DropEdge,
  orientation: SortableOrientation,
  direction: SortableDirection,
): SortablePosition {
  if (edge === "inside") return "inside";
  if (orientation === "vertical") return edge === "top" ? "before" : "after";
  const before = direction === "rtl" ? "right" : "left";
  return edge === before ? "before" : "after";
}

/**
 * Compute the insertion index for moving `fromIndex` relative to `overIndex`.
 *
 * `"inside"` moves report the receiving item's index unchanged.
 */
export function computeToIndex(
  fromIndex: number,
  overIndex: number,
  position: SortablePosition,
): number {
  if (position === "inside" || fromIndex === overIndex) return overIndex;
  if (fromIndex < overIndex) return position === "before" ? overIndex - 1 : overIndex;
  return position === "before" ? overIndex : overIndex + 1;
}

/** Signed index step for one arrow key, or `null` when the key is not owned. */
export function keyboardDelta(
  key: string,
  orientation: SortableOrientation,
  direction: SortableDirection,
  columns: number,
): number | null {
  const forward = direction === "rtl" ? "ArrowLeft" : "ArrowRight";
  const backward = direction === "rtl" ? "ArrowRight" : "ArrowLeft";
  if (orientation === "vertical") {
    if (key === "ArrowUp") return -1;
    if (key === "ArrowDown") return 1;
    return null;
  }
  if (orientation === "grid") {
    if (key === "ArrowUp") return -columns;
    if (key === "ArrowDown") return columns;
  }
  if (key === backward) return -1;
  if (key === forward) return 1;
  return null;
}

/** Dispatch one lifecycle snapshot to the matching consumer callback. */
export function dispatchSortableEvent(options: SortableOptions, event: SortableEvent): void {
  if (event.type === "sortstart") options.onSortStart?.(event as SortableEvent<"sortstart">);
  else if (event.type === "sortpreview") {
    options.onSortPreview?.(event as SortableEvent<"sortpreview">);
  } else if (event.type === "sortcommit") {
    options.onSortCommit?.(event as SortableEvent<"sortcommit">);
  } else options.onSortCancel?.(event as SortableEvent<"sortcancel">);
}

/** Projected list geometry for one drag context. */
export interface SortableProjection {
  readonly currentIndex: number;
  readonly overIndex: number;
  readonly toIndex: number;
  readonly position: SortablePosition | null;
}

interface DragContextSlice {
  readonly edge: DropEdge | null;
  readonly sourceKey: string;
  readonly targetKey: string | null;
}

/** Project a drag-and-drop context onto sortable indexes and positions. */
export function projectDragContext(
  keys: readonly string[],
  orientation: SortableOrientation,
  direction: SortableDirection,
  dragContext: DragContextSlice,
): SortableProjection {
  const currentIndex = Math.max(keys.indexOf(dragContext.sourceKey), 0);
  const overIndex = dragContext.targetKey === null ? -1 : keys.indexOf(dragContext.targetKey);
  const position =
    dragContext.edge === null ? null : positionForEdge(dragContext.edge, orientation, direction);
  const toIndex =
    overIndex >= 0 && position !== null
      ? computeToIndex(currentIndex, overIndex, position)
      : currentIndex;
  return { currentIndex, overIndex, toIndex, position };
}

interface AnnouncementSlice extends DragContextSlice {
  readonly pointerType: DragPointerType;
  readonly sourceLabel: string;
  readonly targetLabel: string | null;
}

/** Build the sortable announcement context for one drag announcement phase. */
export function sortableContextFor(
  phase: "cancel" | "drop" | "grab" | "move",
  dragContext: AnnouncementSlice,
  projection: SortableProjection,
  count: number,
  originIndex: number | null,
): SortableAnnouncementContext {
  const { currentIndex, toIndex, position } = projection;
  const settled = phase === "cancel" || phase === "drop";
  return {
    pointerType: dragContext.pointerType,
    key: dragContext.sourceKey,
    label: dragContext.sourceLabel,
    fromIndex: settled ? (originIndex ?? currentIndex) : currentIndex,
    toIndex: phase === "cancel" ? (originIndex ?? currentIndex) : toIndex,
    count,
    overKey: phase === "cancel" ? null : dragContext.targetKey,
    overLabel: phase === "cancel" ? null : dragContext.targetLabel,
    position: phase === "cancel" ? null : position,
  };
}

function human(index: number): number {
  return index + 1;
}

/** Built-in English announcement builders; consumers override to localize. */
export const defaultSortableAnnouncements: Required<SortableAnnouncements> = Object.freeze({
  grab: (context: SortableAnnouncementContext) => {
    const start = `Picked up ${context.label}, position ${human(context.fromIndex)} of ${context.count}.`;
    return context.pointerType === "keyboard"
      ? `${start} Use the arrow keys to move, Enter to drop, Escape to cancel.`
      : start;
  },
  move: (context: SortableAnnouncementContext) =>
    context.position === "inside"
      ? `${context.label} placed inside ${context.overLabel ?? "the item"}.`
      : `${context.label} moved to position ${human(context.toIndex)} of ${context.count}.`,
  drop: (context: SortableAnnouncementContext) =>
    context.position === "inside"
      ? `${context.label} dropped inside ${context.overLabel ?? "the item"}.`
      : `${context.label} dropped, final position ${human(context.toIndex)} of ${context.count}.`,
  cancel: (context: SortableAnnouncementContext) =>
    `Sorting canceled. ${context.label} returned to position ${human(context.toIndex)} of ${context.count}.`,
});
