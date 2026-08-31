import { toValue } from "vue";

import type {
  DragAndDropOptions,
  DragSourceOptions,
  DropTargetOptions,
} from "./drag-and-drop-controller-types.ts";
import type {
  DragEventType,
  DragLifecycleEvent,
  DragPayload,
  DragPointerType,
  DropEdge,
  DropIndicatorState,
  DropTargetEvent,
  DropTargetEventType,
  DropTargetRect,
} from "./drag-and-drop-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_DRAG_AND_DROP_OPTION";
const dropEdges = new Set<DropEdge>(["bottom", "inside", "left", "right", "top"]);

export interface Point {
  readonly x: number;
  readonly y: number;
}

/** Resolve and validate a reactive boolean option for JavaScript consumers. */
export function readBoolean(value: MaybeBoolean, name: string, defaultValue = false): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return defaultValue;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return resolved;
}

type MaybeBoolean = DragAndDropOptions["isDisabled"];

/** Resolve a finite non-negative pixel distance with an explicit default. */
export function readDistance(
  value: DragAndDropOptions["startDistance"],
  name: string,
  defaultValue: number,
  minimum = 0,
): number {
  const resolved = toValue(value) ?? defaultValue;
  if (typeof resolved !== "number" || !Number.isFinite(resolved) || resolved < minimum) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: ${name} must resolve to a finite number >= ${minimum}`,
    );
  }
  return resolved;
}

/** Resolve an announcement label falling back to the owning key. */
export function readLabel(value: DragSourceOptions["label"], key: string): string {
  const resolved = toValue(value) ?? key;
  if (typeof resolved !== "string") {
    throw new TypeError(`${invalidOptionDiagnostic}: label must resolve to a string`);
  }
  return resolved;
}

/** Validate one callback slot eagerly so setup failures never install listeners. */
export function validateCallbacks(options: object, names: readonly string[]): void {
  for (const name of names) {
    const callback = (options as Record<string, unknown>)[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
}

/** Validate a registration key and its uniqueness inside one controller. */
export function validateKey(
  key: unknown,
  existing: ReadonlySet<string> | ReadonlyMap<string, unknown>,
): string {
  if (typeof key !== "string" || key.length === 0) {
    throw new TypeError(`${invalidOptionDiagnostic}: key must be a non-empty string`);
  }
  if (existing.has(key)) {
    throw new TypeError(`${invalidOptionDiagnostic}: duplicate key "${key}"`);
  }
  return key;
}

/** Validate the closed drop-edge union without accepting mistyped JavaScript. */
export function validateEdges(edges: DropTargetOptions["edges"]): readonly DropEdge[] {
  if (edges === undefined) return ["inside"];
  if (!Array.isArray(edges) || edges.length === 0 || edges.some((edge) => !dropEdges.has(edge))) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: edges must be a non-empty array of drop edges`,
    );
  }
  return [...edges];
}

/** Normalize a measured rectangle, rejecting rects that cannot own a drop. */
export function normalizeRect(
  rect: DropTargetRect | DOMRectReadOnly | null | undefined,
): DropTargetRect | null {
  if (!rect) return null;
  const { top, left, right, bottom } = rect;
  if (![top, left, right, bottom].every(Number.isFinite)) return null;
  if (right < left || bottom < top) return null;
  return { top, left, right, bottom };
}

/** Measure one element through its override or the live layout. */
export function measureRect(
  element: Element | null | undefined,
  getRect: (() => DropTargetRect | DOMRectReadOnly | null | undefined) | undefined,
): DropTargetRect | null {
  if (getRect) return normalizeRect(getRect());
  if (!element || typeof element.getBoundingClientRect !== "function") return null;
  return normalizeRect(element.getBoundingClientRect());
}

/** Whether a client point lies inside an axis-aligned rectangle. */
export function containsPoint(rect: DropTargetRect, point: Point): boolean {
  return (
    point.x >= rect.left && point.x <= rect.right && point.y >= rect.top && point.y <= rect.bottom
  );
}

/** Chebyshev distance gate deciding when an armed pointer becomes a session. */
export function exceedsDistance(origin: Point, point: Point, threshold: number): boolean {
  return Math.max(Math.abs(point.x - origin.x), Math.abs(point.y - origin.y)) >= threshold;
}

/**
 * Resolve the drop edge for a point inside a rectangle.
 *
 * When `"inside"` is allowed together with directional edges, the central half
 * of each constrained axis resolves to `"inside"`; otherwise the nearest
 * allowed edge on the dominant axis wins.
 */
export function resolveEdge(
  rect: DropTargetRect,
  point: Point,
  edges: readonly DropEdge[],
): DropEdge {
  const allowed = new Set(edges);
  const width = Math.max(rect.right - rect.left, 1);
  const height = Math.max(rect.bottom - rect.top, 1);
  const relativeX = (point.x - rect.left) / width - 0.5;
  const relativeY = (point.y - rect.top) / height - 0.5;
  const horizontal = allowed.has("left") || allowed.has("right");
  const vertical = allowed.has("top") || allowed.has("bottom");
  if (allowed.has("inside")) {
    const insideX = !horizontal || Math.abs(relativeX) <= 0.25;
    const insideY = !vertical || Math.abs(relativeY) <= 0.25;
    if ((insideX && insideY) || (!horizontal && !vertical)) return "inside";
  }
  const preferHorizontal = horizontal && (!vertical || Math.abs(relativeX) >= Math.abs(relativeY));
  const axisEdges: readonly [DropEdge, DropEdge] = preferHorizontal
    ? ["left", "right"]
    : ["top", "bottom"];
  const negative = preferHorizontal ? relativeX < 0 : relativeY < 0;
  const preferred = negative ? axisEdges[0] : axisEdges[1];
  const opposite = negative ? axisEdges[1] : axisEdges[0];
  if (allowed.has(preferred)) return preferred;
  if (allowed.has(opposite)) return opposite;
  return edges[0] ?? "inside";
}

/** Build indicator geometry: a placeholder rect plus a collapsed edge line. */
export function indicatorFor(
  targetKey: string,
  edge: DropEdge,
  rect: DropTargetRect | null,
): DropIndicatorState {
  let line: DropTargetRect | null = null;
  if (rect && edge !== "inside") {
    if (edge === "top") line = { ...rect, bottom: rect.top };
    else if (edge === "bottom") line = { ...rect, top: rect.bottom };
    else if (edge === "left") line = { ...rect, right: rect.left };
    else line = { ...rect, left: rect.right };
  }
  return Object.freeze({ targetKey, edge, rect, line });
}

/** Sort elements into document order, treating detached elements as later. */
export function compareDocumentOrder(left: Element, right: Element): number {
  if (left === right) return 0;
  const position = left.compareDocumentPosition(right);
  if (position & 4) return -1; // DOCUMENT_POSITION_FOLLOWING
  if (position & 2) return 1; // DOCUMENT_POSITION_PRECEDING
  return 0;
}

export interface HitCandidate {
  readonly element: Element | null;
  readonly rect: DropTargetRect;
}

/**
 * Overlay-safe hit test: measured rectangles are compared directly, so drag
 * previews and overlays can never mask a target. Nested ownership prefers the
 * innermost target by DOM containment, then the smallest area.
 */
export function hitTest<Candidate extends HitCandidate>(
  candidates: readonly Candidate[],
  point: Point,
): Candidate | null {
  const containing = candidates.filter((candidate) => containsPoint(candidate.rect, point));
  let winner: Candidate | null = null;
  let winnerDepth = -1;
  let winnerArea = Number.POSITIVE_INFINITY;
  for (const candidate of containing) {
    const depth = containing.filter(
      (other) =>
        other !== candidate &&
        other.element !== null &&
        candidate.element !== null &&
        other.element.contains(candidate.element),
    ).length;
    const area =
      (candidate.rect.right - candidate.rect.left) * (candidate.rect.bottom - candidate.rect.top);
    if (depth > winnerDepth || (depth === winnerDepth && area < winnerArea)) {
      winner = candidate;
      winnerDepth = depth;
      winnerArea = area;
    }
  }
  return winner;
}

/** Build a frozen lifecycle snapshot for controller-level callbacks. */
export function createDragEvent<Data, Type extends DragEventType>(
  type: Type,
  pointerType: DragPointerType,
  sourceKey: string,
  payload: DragPayload<Data> | null,
  targetKey: string | null,
  edge: DropEdge | null,
  point: Point | null,
  originalEvent: Event | null,
  isCanceled = false,
): DragLifecycleEvent<Data, Type> {
  return Object.freeze({
    type,
    pointerType,
    sourceKey,
    payload,
    targetKey,
    edge,
    x: point?.x ?? null,
    y: point?.y ?? null,
    originalEvent,
    isCanceled,
  });
}

/** Build a frozen target-scoped snapshot for drop-target callbacks. */
export function createDropTargetEvent<Data>(
  type: DropTargetEventType,
  targetKey: string,
  sourceKey: string,
  pointerType: DragPointerType,
  payload: DragPayload<Data> | null,
  edge: DropEdge | null,
  point: Point | null,
  originalEvent: Event | null,
): DropTargetEvent<Data> {
  return Object.freeze({
    type,
    targetKey,
    sourceKey,
    pointerType,
    payload,
    edge,
    x: point?.x ?? null,
    y: point?.y ?? null,
    originalEvent,
  });
}
