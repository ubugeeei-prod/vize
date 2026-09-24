import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Frame scheduling capability used by {@link useFps}. */
export interface AnimationFrameHost {
  /** Schedule a callback before the next repaint. */
  readonly requestAnimationFrame: (callback: FrameRequestCallback) => number;
  /** Cancel a scheduled frame. */
  readonly cancelAnimationFrame: (handle: number) => void;
}

/** Options for {@link useFps}. */
export interface UseFpsOptions {
  /**
   * Number of frames averaged per update.
   *
   * @default 10
   */
  readonly every?: number;

  /**
   * Start measuring during composable creation.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Reactive frame-scheduling capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<AnimationFrameHost | null | undefined>;
}

/** Reactive frame-rate state returned by {@link useFps}. */
export interface FpsControls {
  /** Frames per second averaged over the last `every` frames; `0` until measured. */
  readonly fps: Readonly<Ref<number>>;
  /** Whether a frame-scheduling capability is attached. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Whether the frame loop is running. */
  readonly isActive: Readonly<Ref<boolean>>;
  /** Stop the frame loop. */
  readonly pause: () => void;
  /** Restart the frame loop. */
  readonly resume: () => void;
}

/**
 * Measure the rendering frame rate with `requestAnimationFrame`.
 *
 * Uses the frame timestamps passed to the callback (no extra clock) and
 * publishes a rounded average every `every` frames. No frames are requested
 * during server rendering (`fps` stays `0`), and the pending frame is
 * cancelled when paused or when the owning reactive scope stops.
 *
 * @param options Averaging window, start mode, and frame capability.
 * @default options {}
 * @returns Reactive frame rate plus pause/resume controls.
 */
export function useFps(options: UseFpsOptions = {}): FpsControls {
  const every = Math.max(1, Math.floor(options.every ?? 10));
  const fps = ref(0);
  const isSupported = ref(false);
  const isActive = ref(options.immediate ?? true);

  const stop = watch(
    [() => (options.host === undefined ? browserFrameHost() : toValue(options.host)), isActive],
    ([host, active], _previous, onCleanup) => {
      isSupported.value = Boolean(host);
      if (!host || !active) return;
      let handle = 0;
      let first: number | undefined;
      let frames = 0;
      const tick = (time: number): void => {
        if (first === undefined) {
          first = time;
        } else {
          frames += 1;
          if (frames >= every) {
            const elapsed = time - first;
            if (elapsed > 0) fps.value = Math.round((1000 * frames) / elapsed);
            first = time;
            frames = 0;
          }
        }
        handle = host.requestAnimationFrame(tick);
      };
      handle = host.requestAnimationFrame(tick);
      onCleanup(() => host.cancelAnimationFrame(handle));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return {
    fps: readonly(fps),
    isSupported: readonly(isSupported),
    isActive: readonly(isActive),
    pause: () => {
      isActive.value = false;
    },
    resume: () => {
      isActive.value = true;
    },
  };
}

function browserFrameHost(): AnimationFrameHost | undefined {
  return typeof window !== "undefined" && typeof window.requestAnimationFrame === "function"
    ? window
    : undefined;
}
