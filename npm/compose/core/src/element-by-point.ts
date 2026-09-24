import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Document-like capability used by {@link useElementByPoint}. */
export interface ElementByPointHost {
  /** Topmost element at a point. */
  elementFromPoint(x: number, y: number): Element | null;
  /** Every element at a point, topmost first. */
  elementsFromPoint(x: number, y: number): Element[];
}

/** Frame scheduler used for continuous polling. */
export interface ElementByPointFrameHost {
  /** Schedule a callback before the next repaint. */
  readonly requestAnimationFrame: (callback: FrameRequestCallback) => number;
  /** Cancel a scheduled frame. */
  readonly cancelAnimationFrame: (handle: number) => void;
}

/** Options for {@link useElementByPoint}. */
export interface UseElementByPointOptions {
  /** Horizontal client coordinate. */
  readonly x: MaybeRefOrGetter<number>;

  /** Vertical client coordinate. */
  readonly y: MaybeRefOrGetter<number>;

  /**
   * Also re-query every animation frame (the element under a fixed point can
   * change when the page scrolls or animates).
   *
   * @default false
   */
  readonly poll?: boolean;

  /**
   * Reactive document capability for alternate runtimes and tests.
   *
   * @default globalThis.document when available
   */
  readonly host?: MaybeRefOrGetter<ElementByPointHost | null | undefined>;

  /**
   * Frame scheduler used when `poll` is enabled.
   *
   * @default globalThis.window when available
   */
  readonly frameHost?: MaybeRefOrGetter<ElementByPointFrameHost | null | undefined>;
}

/** Reactive state returned by {@link useElementByPoint}. */
export interface ElementByPointControls {
  /** Whether a document capability is attached. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Topmost element at the point. */
  readonly element: Readonly<ShallowRef<Element | null>>;
  /** Every element at the point, topmost first. */
  readonly elements: Readonly<ShallowRef<readonly Element[]>>;
  /** Re-query immediately. */
  readonly update: () => void;
}

/**
 * Track the element(s) under a client coordinate.
 *
 * Re-queries whenever `x`/`y` change and, with `poll`, on every animation
 * frame until the owning scope stops. Server renders expose `null` and an
 * empty list.
 *
 * @param options Coordinates, polling, and capabilities.
 * @returns Reactive topmost element, element stack, and a manual update.
 */
export function useElementByPoint(options: UseElementByPointOptions): ElementByPointControls {
  const isSupported = ref(false);
  const element = shallowRef<Element | null>(null);
  const elements = shallowRef<readonly Element[]>([]);
  const host = (): ElementByPointHost | null | undefined =>
    options.host === undefined ? browserPointHost() : toValue(options.host);

  const update = (): void => {
    const current = host();
    isSupported.value = Boolean(current);
    if (!current) {
      element.value = null;
      elements.value = [];
      return;
    }
    const x = toValue(options.x);
    const y = toValue(options.y);
    element.value = current.elementFromPoint(x, y);
    elements.value = current.elementsFromPoint(x, y);
  };

  const stopWatch = watch(() => [host(), toValue(options.x), toValue(options.y)] as const, update, {
    immediate: true,
  });
  const stopPolling = options.poll
    ? watch(
        () => (options.frameHost === undefined ? browserFrameHost() : toValue(options.frameHost)),
        (frames, _previous, onCleanup) => {
          if (!frames) return;
          let handle = 0;
          const tick = (): void => {
            update();
            handle = frames.requestAnimationFrame(tick);
          };
          handle = frames.requestAnimationFrame(tick);
          onCleanup(() => frames.cancelAnimationFrame(handle));
        },
        { immediate: true },
      )
    : undefined;
  tryOnScopeDispose(() => {
    stopWatch.stop();
    stopPolling?.stop();
  });

  return { isSupported: readonly(isSupported), element, elements, update };
}

function browserPointHost(): ElementByPointHost | undefined {
  return typeof document !== "undefined" ? document : undefined;
}

function browserFrameHost(): ElementByPointFrameHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
