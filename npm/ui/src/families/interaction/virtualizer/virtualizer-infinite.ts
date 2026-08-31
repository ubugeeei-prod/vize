/** Bidirectional infinite loading over a virtualizer range, with stale-result cancellation. */

import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import {
  disposedDiagnostic,
  invalidOptionDiagnostic,
  setupDiagnostic,
} from "./virtualizer-options.ts";
import type { VirtualRange } from "./virtualizer-types.ts";

/** Loading direction relative to item indexes. */
export type InfiniteLoadDirection = "backward" | "forward";

/** Lifecycle state for one loading direction. */
export type InfiniteLoadStatus = "idle" | "loading";

/** Context handed to one load callback invocation. */
export interface InfiniteLoadContext {
  /** Direction being loaded: `backward` toward index `0`, `forward` past the end. */
  readonly direction: InfiniteLoadDirection;

  /** Aborted when this invocation becomes stale; its result must be discarded. */
  readonly signal: AbortSignal;
}

/** Options shared by {@link createInfiniteLoader} and {@link useInfiniteLoader}. */
export interface InfiniteLoaderOptions {
  /** Visible range of the driving virtualizer, usually `controller.range`. */
  readonly range: MaybeRefOrGetter<VirtualRange | null | undefined>;

  /** Current collection size. Reactive values re-evaluate loading proximity. */
  readonly count: MaybeRefOrGetter<number>;

  /**
   * Items from either collection edge at which loading starts.
   *
   * @default 4
   */
  readonly threshold?: MaybeRefOrGetter<number | undefined>;

  /**
   * Whether earlier items remain to load before index `0`.
   *
   * @default false
   */
  readonly canLoadBackward?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Whether later items remain to load past the current end.
   *
   * @default false
   */
  readonly canLoadForward?: MaybeRefOrGetter<boolean | undefined>;

  /** Load earlier items. Resolve after the collection is updated. */
  readonly onLoadBackward?: (context: InfiniteLoadContext) => Promise<void> | void;

  /** Load later items. Resolve after the collection is updated. */
  readonly onLoadForward?: (context: InfiniteLoadContext) => Promise<void> | void;

  /** Called when a non-stale load rejects. Without it, the rejection is rethrown. */
  readonly onLoadError?: (direction: InfiniteLoadDirection, error: unknown) => void;
}

/** Bidirectional infinite loading controller. */
export interface InfiniteLoaderController {
  /** Loading state toward index `0`. */
  readonly backwardStatus: Readonly<ShallowRef<InfiniteLoadStatus>>;

  /** Loading state past the current end. */
  readonly forwardStatus: Readonly<ShallowRef<InfiniteLoadStatus>>;

  /** Re-evaluate both directions immediately. */
  readonly check: () => void;

  /** Abort in-flight loads so their eventual results are discarded as stale. */
  readonly cancel: (direction?: InfiniteLoadDirection) => void;

  /** Cancel everything and freeze the controller. */
  readonly dispose: () => void;
}

interface DirectionState {
  readonly status: ShallowRef<InfiniteLoadStatus>;
  token: number;
  aborter: AbortController | null;
}

function readThreshold(value: InfiniteLoaderOptions["threshold"]): number {
  const resolved = toValue(value) ?? 4;
  if (!Number.isInteger(resolved) || resolved < 0) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: threshold must resolve to a non-negative integer`,
    );
  }
  return resolved;
}

/** Create an SSR-safe bidirectional infinite loading controller. */
export function createInfiniteLoader(options: InfiniteLoaderOptions): InfiniteLoaderController {
  for (const name of ["onLoadBackward", "onLoadForward", "onLoadError"] as const) {
    if (options[name] !== undefined && typeof options[name] !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.threshold !== "function") readThreshold(options.threshold);

  const states: Record<InfiniteLoadDirection, DirectionState> = {
    backward: { status: shallowRef("idle"), token: 0, aborter: null },
    forward: { status: shallowRef("idle"), token: 0, aborter: null },
  };
  let disposed = false;

  const assertUsable = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the loader has been disposed`);
  };

  const shouldLoad = (direction: InfiniteLoadDirection): boolean => {
    const enabled = toValue(
      direction === "backward" ? options.canLoadBackward : options.canLoadForward,
    );
    if (enabled !== true) return false;
    const count = toValue(options.count);
    const range = toValue(options.range) ?? null;
    const threshold = readThreshold(options.threshold);
    if (range === null) return direction === "forward" && count === 0;
    return direction === "backward"
      ? range.startIndex <= threshold
      : range.endIndex >= count - 1 - threshold;
  };

  function settle(state: DirectionState, token: number): boolean {
    if (disposed || token !== state.token || state.aborter?.signal.aborted) return false;
    state.aborter = null;
    state.status.value = "idle";
    return true;
  }

  function trigger(direction: InfiniteLoadDirection): void {
    const state = states[direction];
    const load = direction === "backward" ? options.onLoadBackward : options.onLoadForward;
    if (!load || state.status.value === "loading" || !shouldLoad(direction)) return;

    const token = ++state.token;
    const aborter = new AbortController();
    state.aborter = aborter;
    state.status.value = "loading";
    const context: InfiniteLoadContext = Object.freeze({ direction, signal: aborter.signal });

    let result: Promise<void> | void;
    try {
      result = load(context);
    } catch (error) {
      if (settle(state, token)) {
        if (!options.onLoadError) throw error;
        options.onLoadError(direction, error);
      }
      return;
    }
    Promise.resolve(result).then(
      () => {
        if (settle(state, token)) check();
      },
      (error: unknown) => {
        if (!settle(state, token)) return;
        if (options.onLoadError) options.onLoadError(direction, error);
        else throw error;
      },
    );
  }

  function check(): void {
    if (disposed) return;
    trigger("backward");
    trigger("forward");
  }

  function cancel(direction?: InfiniteLoadDirection): void {
    for (const name of ["backward", "forward"] as const) {
      if (direction !== undefined && direction !== name) continue;
      const state = states[name];
      state.token++;
      state.aborter?.abort();
      state.aborter = null;
      state.status.value = "idle";
    }
  }

  const stopWatch = watch(
    () => {
      const range = toValue(options.range) ?? null;
      return [
        range?.startIndex ?? -1,
        range?.endIndex ?? -1,
        toValue(options.count),
        toValue(options.canLoadBackward) === true,
        toValue(options.canLoadForward) === true,
      ] as const;
    },
    () => check(),
    { flush: "sync" },
  );

  check();

  return Object.freeze({
    backwardStatus: shallowReadonly(states.backward.status),
    forwardStatus: shallowReadonly(states.forward.status),
    check: () => {
      assertUsable();
      check();
    },
    cancel: (direction?: InfiniteLoadDirection) => {
      assertUsable();
      cancel(direction);
    },
    dispose() {
      if (disposed) return;
      cancel();
      disposed = true;
      stopWatch();
    },
  });
}

/** Create an infinite loader disposed with the current Vue effect scope. */
export function useInfiniteLoader(options: InfiniteLoaderOptions): InfiniteLoaderController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createInfiniteLoader(options);
  onScopeDispose(controller.dispose);
  return controller;
}
