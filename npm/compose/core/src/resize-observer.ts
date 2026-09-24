import { computed, readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, WatchHandle } from "vue";

import { resolveElement, resolveElements } from "./element-target.ts";
import type { MaybeElementTarget, MaybeElementTargets } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Runtime capability that supplies the `ResizeObserver` constructor. */
export interface ResizeObserverHost {
  /** `ResizeObserver` constructor, absent in unsupported runtimes. */
  readonly ResizeObserver?: new (callback: ResizeObserverCallback) => ResizeObserver;
}

/** Options for {@link useResizeObserver}. */
export interface UseResizeObserverOptions {
  /**
   * Box model reported by the observer.
   *
   * @default "content-box"
   */
  readonly box?: ResizeObserverBoxOptions;

  /**
   * Reactive constructor capability for alternate runtimes and tests.
   *
   * @default globalThis when it provides `ResizeObserver`
   */
  readonly host?: MaybeRefOrGetter<ResizeObserverHost | null | undefined>;

  /**
   * Target resolution timing. `"post"` observes template refs after mount.
   *
   * @default "post"
   */
  readonly flush?: "pre" | "post" | "sync";
}

/** Controls returned by {@link useResizeObserver}. */
export interface ResizeObserverControls {
  /** Whether the host provides `ResizeObserver`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;

  /** Disconnect the observer and stop following the reactive targets. Idempotent. */
  readonly stop: () => void;
}

/**
 * Observe size changes of one or more reactive elements.
 *
 * A single observer is (re)created whenever the resolved targets, the box
 * option, or the host change; unresolved targets are skipped. The observer is
 * disconnected when the owning reactive scope stops or `stop` is called.
 * During server rendering no host resolves, `isSupported` stays `false`, and
 * the callback never runs.
 *
 * @param targets Reactive element target or list of targets.
 * @param callback Native `ResizeObserver` callback.
 * @param options Box model, runtime capability, and watcher timing.
 * @default options {}
 * @returns Support flag and stop control.
 */
export function useResizeObserver(
  targets: MaybeElementTargets,
  callback: ResizeObserverCallback,
  options: UseResizeObserverOptions = {},
): ResizeObserverControls {
  const isSupported = ref(false);
  let stopWatch: WatchHandle | undefined = watch(
    () =>
      [
        options.host === undefined ? browserResizeObserverHost() : toValue(options.host),
        resolveElements(targets),
      ] as const,
    ([host, elements], _previous, onCleanup) => {
      const Observer = host?.ResizeObserver;
      isSupported.value = Observer !== undefined;
      if (!Observer || elements.length === 0) return;
      const observer = new Observer(callback);
      const observeOptions: ResizeObserverOptions = { box: options.box ?? "content-box" };
      for (const element of elements) observer.observe(element, observeOptions);
      onCleanup(() => observer.disconnect());
    },
    { immediate: true, flush: options.flush ?? "post" },
  );

  const stop = (): void => {
    stopWatch?.stop();
    stopWatch = undefined;
  };
  tryOnScopeDispose(stop);
  return { isSupported: readonly(isSupported), stop };
}

/** Element dimensions reported by {@link useElementSize}. */
export interface ElementSize {
  /** Inline size in CSS pixels. */
  readonly width: number;
  /** Block size in CSS pixels. */
  readonly height: number;
}

/** Options for {@link useElementSize}. */
export interface UseElementSizeOptions extends UseResizeObserverOptions {
  /**
   * Size exposed before the first observation and during server rendering.
   * Keep it identical on server and client to avoid hydration mismatches.
   *
   * @default { width: 0, height: 0 }
   */
  readonly initialSize?: ElementSize;
}

/** Reactive size returned by {@link useElementSize}. */
export interface ElementSizeControls extends ResizeObserverControls {
  /** Observed inline size in CSS pixels. */
  readonly width: Readonly<Ref<number>>;
  /** Observed block size in CSS pixels. */
  readonly height: Readonly<Ref<number>>;
}

/**
 * Track the rendered size of a reactive element.
 *
 * Reads the requested box from `contentBoxSize`/`borderBoxSize` (summing
 * fragments) and falls back to `contentRect` for older engines. When the
 * target unmounts the size resets to the initial value so stale geometry never
 * leaks into the next render. Server renders expose `initialSize`.
 *
 * @param target Reactive element target.
 * @param options Initial size, box model, capability, and watcher timing.
 * @default options {}
 * @returns Reactive width/height plus observer controls.
 */
export function useElementSize(
  target: MaybeElementTarget,
  options: UseElementSizeOptions = {},
): ElementSizeControls {
  const initial = options.initialSize ?? { width: 0, height: 0 };
  const width = ref(initial.width);
  const height = ref(initial.height);
  const box = options.box ?? "content-box";

  const controls = useResizeObserver(
    target,
    (entries) => {
      const entry = entries.at(-1);
      if (!entry) return;
      const sizes =
        box === "border-box"
          ? entry.borderBoxSize
          : box === "content-box"
            ? entry.contentBoxSize
            : entry.devicePixelContentBoxSize;
      if (sizes && sizes.length > 0) {
        let inline = 0;
        let block = 0;
        for (const size of sizes) {
          inline += size.inlineSize;
          block += size.blockSize;
        }
        width.value = inline;
        height.value = block;
        return;
      }
      width.value = entry.contentRect.width;
      height.value = entry.contentRect.height;
    },
    options,
  );

  const mounted = computed(() => resolveElement(target) !== null);
  const stopReset = watch(mounted, (isMounted) => {
    if (isMounted) return;
    width.value = initial.width;
    height.value = initial.height;
  });
  const stop = (): void => {
    stopReset.stop();
    controls.stop();
  };
  tryOnScopeDispose(stop);

  return {
    width: readonly(width),
    height: readonly(height),
    isSupported: controls.isSupported,
    stop,
  };
}

function browserResizeObserverHost(): ResizeObserverHost | undefined {
  return typeof ResizeObserver === "function" ? globalThis : undefined;
}
