import { computed, readonly, ref, shallowReactive, shallowReadonly, toValue, watch } from "vue";
import type { ComponentPublicInstance, MaybeRefOrGetter, Ref, WritableComputedRef } from "vue";

import { isElementNode, resolveElement } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Scroll extents shared by elements and the document root element. */
export interface ScrollMetrics {
  /** Total scrollable width. */
  readonly scrollWidth: number;
  /** Total scrollable height. */
  readonly scrollHeight: number;
  /** Visible width without scrollbars. */
  readonly clientWidth: number;
  /** Visible height without scrollbars. */
  readonly clientHeight: number;
}

/** Window-like scroll container accepted by {@link useScroll}. */
export interface ScrollWindowHost extends EventTarget {
  /** Horizontal document scroll offset. */
  readonly scrollX: number;
  /** Vertical document scroll offset. */
  readonly scrollY: number;
  /** Scroll the document. */
  scrollTo(options: ScrollToOptions): void;
  /** Document whose root element reports the scroll extents. */
  readonly document: { readonly documentElement: ScrollMetrics };
}

/** Value accepted as a scroll container. */
export type ScrollTargetValue =
  | Element
  | ComponentPublicInstance
  | ScrollWindowHost
  | null
  | undefined;

/** Per-edge boolean state. */
export interface ScrollEdges {
  /** Left edge (or inline start in LTR). */
  readonly left: boolean;
  /** Right edge. */
  readonly right: boolean;
  /** Top edge. */
  readonly top: boolean;
  /** Bottom edge. */
  readonly bottom: boolean;
}

/** Per-edge distance thresholds in CSS pixels. */
export interface ScrollOffset {
  /** Distance from the left edge that still counts as arrived. */
  readonly left?: number;
  /** Distance from the right edge that still counts as arrived. */
  readonly right?: number;
  /** Distance from the top edge that still counts as arrived. */
  readonly top?: number;
  /** Distance from the bottom edge that still counts as arrived. */
  readonly bottom?: number;
}

/** Options for {@link useScroll}. */
export interface UseScrollOptions {
  /**
   * Edge tolerances used to compute `arrivedState`.
   *
   * @default { left: 0, right: 0, top: 0, bottom: 0 }
   */
  readonly offset?: ScrollOffset;

  /**
   * Milliseconds without scroll events before `isScrolling` becomes `false`
   * (used when the engine does not fire `scrollend`).
   *
   * @default 200
   */
  readonly idle?: number;

  /**
   * Scroll behavior used by `scrollTo` and the writable `x`/`y` refs.
   *
   * @default "auto"
   */
  readonly behavior?: MaybeRefOrGetter<ScrollBehavior>;

  /**
   * Called for every scroll event after the state has been updated.
   *
   * @default undefined
   */
  readonly onScroll?: (event: Event) => void;

  /**
   * Called once scrolling settles.
   *
   * @default undefined
   */
  readonly onStop?: (event: Event) => void;

  /**
   * Owns the idle timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Target resolution timing. `"post"` observes template refs after mount.
   *
   * @default "post"
   */
  readonly flush?: "pre" | "post" | "sync";
}

/** Reactive scroll state returned by {@link useScroll}. */
export interface ScrollControls {
  /** Horizontal offset. Assigning scrolls the target with the configured behavior. */
  readonly x: WritableComputedRef<number>;
  /** Vertical offset. Assigning scrolls the target with the configured behavior. */
  readonly y: WritableComputedRef<number>;
  /** Whether scroll events arrived within the idle window. */
  readonly isScrolling: Readonly<Ref<boolean>>;
  /** Whether each edge has been reached (within `offset`). */
  readonly arrivedState: Readonly<ScrollEdges>;
  /** Direction of the most recent scroll movement; reset when scrolling stops. */
  readonly directions: Readonly<ScrollEdges>;
  /** Re-read offsets and edges without waiting for a scroll event. */
  readonly measure: () => void;
  /** Scroll the target. Missing axes keep their current offset. */
  readonly scrollTo: (options: { readonly left?: number; readonly top?: number }) => void;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

/**
 * Track the scroll offset, edges, and direction of an element or window.
 *
 * During server rendering (or while the target is unresolved) offsets are `0`,
 * the top/left edges report arrived, and the bottom/right edges do not —
 * matching an unscrolled page, so hydration is stable. The scroll listener is
 * passive and removed together with the idle timer when the owning reactive
 * scope stops.
 *
 * @param target Reactive element, component, or window-like target.
 * @param options Edge offsets, idle timing, behavior, and callbacks.
 * @default options {}
 * @returns Writable offsets, activity, edges, directions, and controls.
 */
export function useScroll(
  target: MaybeRefOrGetter<ScrollTargetValue>,
  options: UseScrollOptions = {},
): ScrollControls {
  const offset = options.offset ?? {};
  const idle = options.idle ?? 200;
  const scheduler = options.scheduler ?? defaultScheduler;
  const internalX = ref(0);
  const internalY = ref(0);
  const isScrolling = ref(false);
  const arrivedState = shallowReactive({ left: true, right: false, top: true, bottom: false });
  const directions = shallowReactive({ left: false, right: false, top: false, bottom: false });
  let idleHandle: unknown;
  let hasIdleTimer = false;

  const resolveContainer = (): Element | ScrollWindowHost | null => {
    const value = toValue(target);
    if (isScrollWindow(value)) return value;
    return resolveElement(value);
  };

  const measure = (): void => {
    const container = resolveContainer();
    if (!container) return;
    const metrics = isScrollWindow(container) ? container.document.documentElement : container;
    const x = isScrollWindow(container) ? container.scrollX : container.scrollLeft;
    const y = isScrollWindow(container) ? container.scrollY : container.scrollTop;

    directions.left = x < internalX.value;
    directions.right = x > internalX.value;
    directions.top = y < internalY.value;
    directions.bottom = y > internalY.value;
    internalX.value = x;
    internalY.value = y;

    const left = Math.abs(x) <= (offset.left ?? 0);
    const top = Math.abs(y) <= (offset.top ?? 0);
    arrivedState.left = left;
    arrivedState.top = top;
    arrivedState.right =
      Math.abs(x) + metrics.clientWidth >= metrics.scrollWidth - (offset.right ?? 0) - 1;
    arrivedState.bottom =
      Math.abs(y) + metrics.clientHeight >= metrics.scrollHeight - (offset.bottom ?? 0) - 1;
  };

  const settle = (event: Event): void => {
    if (!isScrolling.value) return;
    isScrolling.value = false;
    directions.left = false;
    directions.right = false;
    directions.top = false;
    directions.bottom = false;
    options.onStop?.(event);
  };
  const clearIdle = (): void => {
    if (!hasIdleTimer) return;
    scheduler.clearTimeout(idleHandle);
    hasIdleTimer = false;
  };

  const onScroll = (event: Event): void => {
    measure();
    isScrolling.value = true;
    clearIdle();
    idleHandle = scheduler.setTimeout(() => {
      hasIdleTimer = false;
      settle(event);
    }, idle);
    hasIdleTimer = true;
    options.onScroll?.(event);
  };
  const onScrollEnd = (event: Event): void => {
    clearIdle();
    settle(event);
  };

  const stopWatch = watch(
    resolveContainer,
    (container, _previous, onCleanup) => {
      if (!container) return;
      measure();
      const listenerOptions: AddEventListenerOptions = { passive: true };
      container.addEventListener("scroll", onScroll, listenerOptions);
      container.addEventListener("scrollend", onScrollEnd, listenerOptions);
      onCleanup(() => {
        container.removeEventListener("scroll", onScroll);
        container.removeEventListener("scrollend", onScrollEnd);
        clearIdle();
      });
    },
    { immediate: true, flush: options.flush ?? "post" },
  );
  tryOnScopeDispose(() => {
    stopWatch.stop();
    clearIdle();
  });

  const scrollTo = (next: { readonly left?: number; readonly top?: number }): void => {
    const container = resolveContainer();
    if (!container) return;
    container.scrollTo({
      left: next.left ?? internalX.value,
      top: next.top ?? internalY.value,
      behavior: toValue(options.behavior) ?? "auto",
    });
  };

  return {
    x: computed({ get: () => internalX.value, set: (left) => scrollTo({ left }) }),
    y: computed({ get: () => internalY.value, set: (top) => scrollTo({ top }) }),
    isScrolling: readonly(isScrolling),
    arrivedState: shallowReadonly(arrivedState),
    directions: shallowReadonly(directions),
    measure,
    scrollTo,
  };
}

/** Options for {@link useWindowScroll}. */
export interface UseWindowScrollOptions extends UseScrollOptions {
  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<ScrollWindowHost | null | undefined>;
}

/**
 * Track the document scroll position.
 *
 * Equivalent to {@link useScroll} with the window as target, so it shares its
 * SSR-stable defaults and scope-bound cleanup.
 *
 * @param options Window capability plus every {@link useScroll} option.
 * @default options {}
 * @returns The same controls as {@link useScroll}.
 */
export function useWindowScroll(options: UseWindowScrollOptions = {}): ScrollControls {
  const { host, ...scrollOptions } = options;
  return useScroll(
    () => (host === undefined ? browserScrollWindow() : toValue(host)),
    scrollOptions,
  );
}

function isScrollWindow(value: unknown): value is ScrollWindowHost {
  return (
    typeof value === "object" &&
    value !== null &&
    !isElementNode(value) &&
    "scrollX" in value &&
    "document" in value
  );
}

function browserScrollWindow(): ScrollWindowHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
