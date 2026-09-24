import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { useScroll } from "./scroll.ts";
import type { ScrollTargetValue, UseScrollOptions } from "./scroll.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Edge that triggers loading in {@link useInfiniteScroll}. */
export type InfiniteScrollDirection = "top" | "bottom" | "left" | "right";

/** Options for {@link useInfiniteScroll}. */
export interface UseInfiniteScrollOptions extends Omit<UseScrollOptions, "offset"> {
  /**
   * Distance in CSS pixels from the edge that triggers loading.
   *
   * @default 0
   */
  readonly distance?: number;

  /**
   * Edge that triggers loading.
   *
   * @default "bottom"
   */
  readonly direction?: InfiniteScrollDirection;

  /**
   * Return `false` to stop loading (for example once every page is loaded).
   *
   * @default () => true
   */
  readonly canLoadMore?: MaybeRefOrGetter<boolean>;
}

/** Reactive state returned by {@link useInfiniteScroll}. */
export interface InfiniteScrollControls {
  /** Whether `onLoadMore` is running. */
  readonly isLoading: Readonly<Ref<boolean>>;
  /**
   * Re-check the edge and load when it is still within `distance` (call after
   * the list was replaced).
   */
  readonly reset: () => void;
}

/**
 * Load more content when a scroll container reaches an edge.
 *
 * Built on `useScroll`: when the configured edge is within `distance`,
 * `onLoadMore` runs; concurrent calls are prevented, and after each load the
 * edge is re-measured so short pages keep loading until the container
 * overflows or `canLoadMore` returns `false`. Loader errors propagate to the
 * returned promise of the internal run but never leave `isLoading` stuck.
 * No loading happens during server rendering.
 *
 * @param target Reactive scroll container (element, component, or window).
 * @param onLoadMore Loader invoked at the edge.
 * @param options Edge, distance, gating, and {@link useScroll} options.
 * @default options {}
 * @returns Loading flag and a manual re-check.
 */
export function useInfiniteScroll(
  target: MaybeRefOrGetter<ScrollTargetValue>,
  onLoadMore: () => unknown,
  options: UseInfiniteScrollOptions = {},
): InfiniteScrollControls {
  const { distance = 0, direction = "bottom", canLoadMore = true, ...scrollOptions } = options;
  const isLoading = ref(false);
  const scroll = useScroll(target, { ...scrollOptions, offset: { [direction]: distance } });
  let disposed = false;

  const check = async (): Promise<void> => {
    if (disposed || isLoading.value || !toValue(canLoadMore)) return;
    if (!scroll.arrivedState[direction] || !hasContainer(target)) return;
    isLoading.value = true;
    try {
      await onLoadMore();
    } finally {
      isLoading.value = false;
    }
    await Promise.resolve();
    if (disposed) return;
    scroll.measure();
    void check();
  };

  const stop = watch(
    () => [scroll.arrivedState[direction], toValue(canLoadMore)] as const,
    () => void check(),
    { immediate: true },
  );
  tryOnScopeDispose(() => {
    disposed = true;
    stop.stop();
  });

  return {
    isLoading: readonly(isLoading),
    reset: () => {
      scroll.measure();
      void check();
    },
  };
}

function hasContainer(target: MaybeRefOrGetter<ScrollTargetValue>): boolean {
  const value = toValue(target);
  return value !== null && value !== undefined;
}
