import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { useEventListener } from "./event-listener.ts";
import { useResizeObserver } from "./resize-observer.ts";
import type { ResizeObserverHost } from "./resize-observer.ts";
import { tryOnScopeDispose } from "./scope.ts";

/**
 * Window-like capability observed by {@link useElementBounding}: scroll and
 * resize events plus the optional `ResizeObserver` constructor.
 */
export interface ElementBoundingHost extends EventTarget, ResizeObserverHost {}

/** Viewport-relative element geometry in CSS pixels. */
export interface ElementBoundingRect {
  /** Horizontal position of the border box origin. */
  readonly x: number;
  /** Vertical position of the border box origin. */
  readonly y: number;
  /** Distance from the viewport top edge to the border box top edge. */
  readonly top: number;
  /** Distance from the viewport left edge to the border box right edge. */
  readonly right: number;
  /** Distance from the viewport top edge to the border box bottom edge. */
  readonly bottom: number;
  /** Distance from the viewport left edge to the border box left edge. */
  readonly left: number;
  /** Border box width. */
  readonly width: number;
  /** Border box height. */
  readonly height: number;
}

/** Options for {@link useElementBounding}. */
export interface UseElementBoundingOptions {
  /**
   * Reset every value to zero when the target unmounts.
   *
   * @default true
   */
  readonly reset?: boolean;

  /**
   * Re-measure when the window resizes.
   *
   * @default true
   */
  readonly windowResize?: boolean;

  /**
   * Re-measure when any ancestor (or the window) scrolls. Uses a capturing,
   * passive listener so nested scroll containers are covered.
   *
   * @default true
   */
  readonly windowScroll?: boolean;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<ElementBoundingHost | null | undefined>;

  /**
   * Target resolution timing. `"post"` measures template refs after mount.
   *
   * @default "post"
   */
  readonly flush?: "pre" | "post" | "sync";
}

/** Reactive geometry returned by {@link useElementBounding}. */
export type ElementBoundingControls = {
  /** Reactive geometry field; zero during server rendering. */
  readonly [Key in keyof ElementBoundingRect]: Readonly<Ref<number>>;
} & {
  /** Measure the target immediately. Does nothing while it is unresolved. */
  readonly update: () => void;

  /** Stop observing resize, scroll, and target changes. Idempotent. */
  readonly stop: () => void;
};

const boundingKeys = [
  "x",
  "y",
  "top",
  "right",
  "bottom",
  "left",
  "width",
  "height",
] as const satisfies readonly (keyof ElementBoundingRect)[];

/**
 * Track an element's `getBoundingClientRect()` reactively.
 *
 * Measures when the target resolves, when it resizes (via `ResizeObserver`),
 * and — unless disabled — when the window resizes or anything scrolls. Every
 * value is `0` during server rendering and before the first measurement, so
 * the initial render is hydration-stable. Listeners and the observer are
 * released with the owning reactive scope.
 *
 * @param target Reactive element target.
 * @param options Reset, listener, capability, and timing options.
 * @default options {}
 * @returns One readonly ref per geometry field plus `update`/`stop`.
 */
export function useElementBounding(
  target: MaybeElementTarget,
  options: UseElementBoundingOptions = {},
): ElementBoundingControls {
  const { reset = true, windowResize = true, windowScroll = true } = options;
  const flush = options.flush ?? "post";
  const state = {
    x: ref(0),
    y: ref(0),
    top: ref(0),
    right: ref(0),
    bottom: ref(0),
    left: ref(0),
    width: ref(0),
    height: ref(0),
  };
  const host = (): ElementBoundingHost | null | undefined =>
    options.host === undefined ? browserBoundingHost() : toValue(options.host);

  const update = (): void => {
    const element = resolveElement(target);
    if (!element) {
      if (reset) for (const key of boundingKeys) state[key].value = 0;
      return;
    }
    const rect = element.getBoundingClientRect();
    for (const key of boundingKeys) state[key].value = rect[key];
  };

  const observer = useResizeObserver(target, update, { host, flush });
  const stopTarget = watch(() => resolveElement(target), update, { immediate: true, flush });
  const resize = windowResize
    ? useEventListener(host, "resize", update, { passive: true, flush })
    : undefined;
  const scroll = windowScroll
    ? useEventListener(host, "scroll", update, { capture: true, passive: true, flush })
    : undefined;

  const stop = (): void => {
    observer.stop();
    stopTarget.stop();
    resize?.stop();
    scroll?.stop();
  };
  tryOnScopeDispose(stop);

  return {
    x: readonly(state.x),
    y: readonly(state.y),
    top: readonly(state.top),
    right: readonly(state.right),
    bottom: readonly(state.bottom),
    left: readonly(state.left),
    width: readonly(state.width),
    height: readonly(state.height),
    update,
    stop,
  };
}

function browserBoundingHost(): ElementBoundingHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
