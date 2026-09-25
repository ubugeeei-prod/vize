/** Window display mode. */
export type WindowMode = "normal" | "minimized" | "maximized";

/** Resize edge or corner. */
export type WindowEdge = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

/** Position and size in CSS pixels, relative to the manager. */
export interface WindowRect {
  /** Left edge. */
  readonly x: number;
  /** Top edge. */
  readonly y: number;
  /** Width. */
  readonly width: number;
  /** Height. */
  readonly height: number;
}

/** Persisted state of one window. */
export interface WindowEntry extends WindowRect {
  /** Display mode. */
  readonly mode: WindowMode;
}

/** Serializable layout of every window, plus stacking order (last = front). */
export interface WindowLayout {
  /** Window state by id. */
  readonly windows: Readonly<Record<string, WindowEntry>>;
  /** Window ids from back to front. */
  readonly order: readonly string[];
}

/** Size limits of one window. */
export interface WindowConstraints {
  /** Minimum width. */
  readonly minWidth: number;
  /** Minimum height. */
  readonly minHeight: number;
  /** Maximum width. */
  readonly maxWidth: number;
  /** Maximum height. */
  readonly maxHeight: number;
}

/** Size of the manager area. */
export interface WindowBounds {
  /** Area width. */
  readonly width: number;
  /** Area height. */
  readonly height: number;
}

/** An empty layout. */
export const emptyWindowLayout: WindowLayout = Object.freeze({
  windows: Object.freeze({}),
  order: Object.freeze([]),
});

/**
 * Keep a rect inside the bounds when it fits, otherwise pin it to the top-left
 * corner. Unknown bounds (`null`, e.g. during SSR) leave the rect untouched.
 *
 * @param rect Candidate rect.
 * @param bounds Manager size, or `null` when unknown.
 * @returns The clamped rect.
 */
export function clampWindowRect(rect: WindowRect, bounds: WindowBounds | null): WindowRect {
  if (!bounds) return rect;
  return {
    ...rect,
    x: clamp(rect.x, 0, Math.max(0, bounds.width - rect.width)),
    y: clamp(rect.y, 0, Math.max(0, bounds.height - rect.height)),
  };
}

/**
 * Snap a moving rect's edges to the bounds and to other windows' edges that
 * are within `threshold` pixels.
 *
 * @param rect Candidate rect.
 * @param others Rects of the other visible windows.
 * @param bounds Manager size, or `null` when unknown.
 * @param threshold Snap distance; `0` disables snapping.
 * @returns The snapped rect (size unchanged).
 */
export function snapWindowRect(
  rect: WindowRect,
  others: readonly WindowRect[],
  bounds: WindowBounds | null,
  threshold: number,
): WindowRect {
  if (!(threshold > 0)) return rect;
  const verticalLines = [
    ...(bounds ? [0, bounds.width] : []),
    ...others.flatMap((o) => [o.x, o.x + o.width]),
  ];
  const horizontalLines = [
    ...(bounds ? [0, bounds.height] : []),
    ...others.flatMap((o) => [o.y, o.y + o.height]),
  ];
  const x = snapAxis(rect.x, rect.width, verticalLines, threshold);
  const y = snapAxis(rect.y, rect.height, horizontalLines, threshold);
  return { ...rect, x, y };
}

/**
 * Move a rect by a delta, then snap and clamp it.
 *
 * @param rect Current rect.
 * @param deltaX Horizontal movement.
 * @param deltaY Vertical movement.
 * @param others Rects of the other visible windows.
 * @param bounds Manager size, or `null` when unknown.
 * @param threshold Snap distance.
 * @default threshold 0
 * @returns The moved rect.
 */
export function moveWindowRect(
  rect: WindowRect,
  deltaX: number,
  deltaY: number,
  others: readonly WindowRect[],
  bounds: WindowBounds | null,
  threshold = 0,
): WindowRect {
  const moved = { ...rect, x: rect.x + deltaX, y: rect.y + deltaY };
  return clampWindowRect(snapWindowRect(moved, others, bounds, threshold), bounds);
}

/**
 * Resize a rect from an edge or corner, respecting size constraints and
 * keeping the opposite edge fixed.
 *
 * @param rect Current rect.
 * @param edge Dragged edge or corner.
 * @param deltaX Horizontal pointer movement.
 * @param deltaY Vertical pointer movement.
 * @param constraints Size limits.
 * @param bounds Manager size, or `null` when unknown.
 * @returns The resized rect.
 */
export function resizeWindowRect(
  rect: WindowRect,
  edge: WindowEdge,
  deltaX: number,
  deltaY: number,
  constraints: WindowConstraints,
  bounds: WindowBounds | null,
): WindowRect {
  let { x, y, width, height } = rect;
  const maxWidth = Math.min(constraints.maxWidth, bounds ? bounds.width : Number.POSITIVE_INFINITY);
  const maxHeight = Math.min(
    constraints.maxHeight,
    bounds ? bounds.height : Number.POSITIVE_INFINITY,
  );
  if (edge.includes("e")) {
    width = clamp(
      width + deltaX,
      constraints.minWidth,
      bounds ? Math.min(maxWidth, bounds.width - x) : maxWidth,
    );
  }
  if (edge.includes("s")) {
    height = clamp(
      height + deltaY,
      constraints.minHeight,
      bounds ? Math.min(maxHeight, bounds.height - y) : maxHeight,
    );
  }
  if (edge.includes("w")) {
    const right = x + width;
    width = clamp(
      width - deltaX,
      constraints.minWidth,
      bounds ? Math.min(maxWidth, right) : maxWidth,
    );
    x = right - width;
  }
  if (edge.includes("n")) {
    const bottom = y + height;
    height = clamp(
      height - deltaY,
      constraints.minHeight,
      bounds ? Math.min(maxHeight, bottom) : maxHeight,
    );
    y = bottom - height;
  }
  return { x, y, width, height };
}

/**
 * Move `id` to the front of the stacking order (appending unknown ids).
 *
 * @param order Ids from back to front.
 * @param id Window to raise.
 * @returns The new order (the same array when already in front).
 */
export function bringWindowToFront(order: readonly string[], id: string): readonly string[] {
  if (order.at(-1) === id) return order;
  return [...order.filter((entry) => entry !== id), id];
}

/**
 * Parse a stored layout, dropping malformed windows and unknown order ids.
 *
 * @param value JSON text (or `null`).
 * @returns The layout, or `undefined` when the text is not a layout.
 */
export function parseWindowLayout(value: string | null): WindowLayout | undefined {
  if (value === null) return undefined;
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    return undefined;
  }
  if (!isRecord(parsed) || !isRecord(parsed["windows"]) || !Array.isArray(parsed["order"])) {
    return undefined;
  }
  const windows: Record<string, WindowEntry> = {};
  for (const [id, entry] of Object.entries(parsed["windows"])) {
    if (isWindowEntry(entry))
      windows[id] = {
        x: entry.x,
        y: entry.y,
        width: entry.width,
        height: entry.height,
        mode: entry.mode,
      };
  }
  const order = parsed["order"].filter(
    (id): id is string => typeof id === "string" && Object.hasOwn(windows, id),
  );
  return { windows, order: [...new Set(order)] };
}

function snapAxis(
  start: number,
  size: number,
  lines: readonly number[],
  threshold: number,
): number {
  let best = start;
  let bestDistance = threshold + 1;
  for (const line of lines) {
    for (const candidate of [line, line - size]) {
      const distance = Math.abs(candidate - start);
      if (distance <= threshold && distance < bestDistance) {
        best = candidate;
        bestDistance = distance;
      }
    }
  }
  return best;
}

function isWindowEntry(value: unknown): value is WindowEntry {
  return (
    isRecord(value) &&
    ["x", "y", "width", "height"].every((key) => Number.isFinite(value[key])) &&
    (value["mode"] === "normal" || value["mode"] === "minimized" || value["mode"] === "maximized")
  );
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), Math.max(minimum, maximum));
}
