import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** 2D point in the same coordinate space as {@link Rect}. */
export interface Point {
  readonly x: number;
  readonly y: number;
}

/** Axis-aligned box in viewport coordinates. */
export interface Rect {
  readonly height: number;
  readonly width: number;
  readonly x: number;
  readonly y: number;
}

/** Options shared by {@link createPointerGrace} and {@link usePointerGrace}. */
export interface PointerGraceOptions {
  /**
   * Milliseconds to wait after the pointer leaves the polygon before ending.
   *
   * @default 300
   */
  readonly delay?: MaybeRefOrGetter<number | undefined>;

  /** Called once the pointer stays outside the polygon for `delay`. */
  readonly onGraceEnd?: () => void;
}

/** Stateful safe-triangle tracker for hover menus. */
export interface PointerGraceController {
  /** Whether a point lies in the current origin→target polygon or target rect. */
  readonly contains: (point: Point) => boolean;

  /** Release the pending timer. */
  readonly dispose: () => void;

  /** Latest pointer sample used as the triangle origin. */
  readonly handleMove: (point: Point) => void;

  /** Whether a grace timer is currently running. */
  readonly isPending: Readonly<ShallowRef<boolean>>;

  /** Current polygon vertices, or `null` when origin or target is unset. */
  readonly polygon: Readonly<ShallowRef<readonly Point[] | null>>;

  /** Bind the pointer origin used to build the safe triangle. */
  readonly setOrigin: (point: Point | null) => void;

  /** Bind the destination box (usually the floating element). */
  readonly setTarget: (rect: Rect | null) => void;
}
