import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

/** Named minimum widths in CSS pixels (mobile-first). */
export type BreakpointMap<Name extends string = string> = Readonly<Record<Name, number>>;

/** Window-like capability read by {@link useBreakpoint}. */
export interface BreakpointHost extends EventTarget {
  /** Viewport width in CSS pixels. */
  readonly innerWidth: number;
}

/** Options for {@link useBreakpoint}. */
export interface UseBreakpointOptions {
  /**
   * Viewport width assumed during server rendering and hydration. Without it
   * the width is `null` until mount and every comparison is `false`.
   *
   * @default undefined
   */
  readonly ssrWidth?: MaybeRefOrGetter<number | undefined>;

  /**
   * Read the real width during setup instead of after mount. Only
   * hydration-safe when the server rendered the same width.
   *
   * @default false
   */
  readonly immediate?: boolean;

  /**
   * Window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: BreakpointHost | null;
}

/** Reactive breakpoint state returned by {@link useBreakpoint}. */
export interface BreakpointState<Name extends string> {
  /** Current viewport width, or `ssrWidth` until mount (`null` without it). */
  readonly width: Readonly<Ref<number | null>>;
  /** Largest breakpoint whose minimum width is reached, or `null` below all of them. */
  readonly active: ComputedRef<Name | null>;
  /** Whether the viewport is at least `name` wide. */
  readonly isAbove: (name: Name) => boolean;
  /** Whether the viewport is narrower than `name`. */
  readonly isBelow: (name: Name) => boolean;
  /** Whether `lower` ≤ width < `upper`. */
  readonly isBetween: (lower: Name, upper: Name) => boolean;
}

/** Slot props of `ResponsiveSwitch`. */
export interface ResponsiveSwitchSlotState {
  /** Active breakpoint name, or `null` below every breakpoint. */
  readonly active: string | null;
  /** Current (or SSR) viewport width. */
  readonly width: number | null;
}

/** Slot props of `ResponsiveShow`. */
export interface ResponsiveShowSlotState {
  /** Whether the content is currently shown. */
  readonly visible: boolean;
}

/** How `ResponsiveShow` hides content outside its range. */
export type ResponsiveHideMode = "unmount" | "hidden";
