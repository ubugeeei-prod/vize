import { shallowRef, toValue } from "vue";
import type { MaybeRefOrGetter } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { PausableControls } from "./use-interval.ts";

/**
 * Animation-frame host used by {@link useRafFn}.
 *
 * Implement this interface to drive frames from a deterministic test clock,
 * a native display link, or an offscreen renderer.
 */
export interface FrameScheduler {
  /** Requests one frame callback and returns its opaque cancellation handle. */
  readonly requestAnimationFrame: (callback: (timestamp: number) => void) => unknown;

  /** Cancels a handle previously returned by {@link FrameScheduler.requestAnimationFrame}. */
  readonly cancelAnimationFrame: (handle: unknown) => void;
}

/** Timing information passed to every {@link useRafFn} callback. */
export interface RafFrame {
  /** Milliseconds since the previous delivered frame (`0` for the first). */
  readonly delta: number;

  /** High-resolution frame timestamp supplied by the host. */
  readonly timestamp: number;
}

/** Options for {@link useRafFn}. */
export interface UseRafFnOptions {
  /**
   * Start the frame loop as soon as the composable is created.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Maximum callback rate in frames per second. Frames arriving sooner than
   * `1000 / fpsLimit` ms after the last delivered frame are skipped.
   * `undefined` delivers every frame. Reactive.
   *
   * @default undefined
   */
  readonly fpsLimit?: MaybeRefOrGetter<number | undefined>;

  /**
   * Deliver a single frame and then pause.
   *
   * @default false
   */
  readonly once?: boolean;

  /**
   * Requests frames when no browser `window` is available, for example from
   * a native or offscreen frame host passed as `scheduler`.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Frame host. When omitted, `globalThis.requestAnimationFrame` is used if
   * it exists; without one (server rendering, workers without frames) the
   * loop only tracks its requested state.
   *
   * @default globalThis animation-frame functions when available
   */
  readonly scheduler?: FrameScheduler;
}

function resolveHost(scheduler: FrameScheduler | undefined): FrameScheduler | undefined {
  if (scheduler !== undefined) return scheduler;
  if (typeof globalThis.requestAnimationFrame !== "function") return undefined;
  return {
    requestAnimationFrame: (callback) => globalThis.requestAnimationFrame(callback),
    cancelAnimationFrame: (handle) => {
      globalThis.cancelAnimationFrame(handle as number);
    },
  };
}

/**
 * Run a callback on every animation frame, with pause/resume controls and an
 * optional frame-rate cap.
 *
 * The host is resolved lazily on `resume()`, so importing and calling this
 * during server rendering is safe: without a browser `window` (and without
 * {@link UseRafFnOptions.runOnServer}), or without any frame host,
 * `isActive` reports the requested state and no callbacks run, matching the
 * client's initial render. The pending frame is cancelled when the owning reactive scope
 * stops.
 *
 * @example
 * ```ts
 * const { pause } = useRafFn(({ delta }) => step(delta), { fpsLimit: 30 });
 * ```
 *
 * @param callback Invoked with frame timing for every delivered frame.
 * @param options Start, rate-limit, and host policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_RAF_INVALID_FPS_LIMIT` when a
 * resolved `fpsLimit` is not finite and greater than zero.
 * @returns Pause/resume controls.
 */
export function useRafFn(
  callback: (frame: RafFrame) => void,
  options: UseRafFnOptions = {},
): PausableControls {
  const isActive = shallowRef(false);
  let host: FrameScheduler | undefined;
  let handle: unknown;
  let requested = false;
  let previous: number | undefined;

  const readInterval = (): number => {
    const limit = toValue(options.fpsLimit);
    if (limit === undefined) return 0;
    if (!Number.isFinite(limit) || limit <= 0) {
      throw new RangeError(
        `[VIZE_COMPOSE_RAF_INVALID_FPS_LIMIT] fpsLimit must be finite and greater than zero; received ${String(limit)}`,
      );
    }
    return 1_000 / limit;
  };

  readInterval();

  const request = (): void => {
    if (host === undefined) return;
    requested = true;
    handle = host.requestAnimationFrame(loop);
  };

  function loop(timestamp: number): void {
    requested = false;
    if (!isActive.value) return;
    previous ??= timestamp;
    const delta = timestamp - previous;
    const interval = readInterval();
    if (interval > 0 && delta > 0 && delta < interval) {
      request();
      return;
    }
    previous = timestamp;
    callback({ delta, timestamp });
    if (options.once ?? false) {
      pause();
      return;
    }
    request();
  }

  const pause = (): void => {
    isActive.value = false;
    if (requested && host !== undefined) host.cancelAnimationFrame(handle);
    requested = false;
    handle = undefined;
  };

  const resume = (): void => {
    if (isActive.value) return;
    isActive.value = true;
    previous = undefined;
    if (typeof window === "undefined" && !(options.runOnServer ?? false)) return;
    host ??= resolveHost(options.scheduler);
    request();
  };

  if (options.immediate ?? true) resume();

  tryOnScopeDispose(pause);

  return { isActive, pause, resume };
}
