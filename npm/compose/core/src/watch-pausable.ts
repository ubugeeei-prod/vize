import { shallowRef, watch } from "vue";
import type { ShallowRef, WatchCallback, WatchOptions } from "vue";

import type { WatchHelperCallback, WatchSources } from "./watch-source.ts";

/** Controls returned by {@link watchPausable}. */
export interface PausableWatchHandle {
  /** Whether changes are currently delivered to the callback. */
  readonly isActive: Readonly<ShallowRef<boolean>>;

  /** Drop changes until {@link PausableWatchHandle.resume}. Idempotent. */
  readonly pause: () => void;

  /** Deliver changes again. Changes made while paused are not replayed. */
  readonly resume: () => void;

  /** Stop watching permanently. */
  readonly stop: () => void;
}

/** Options for {@link watchPausable}. */
export interface WatchPausableOptions<
  Immediate extends boolean = false,
> extends WatchOptions<Immediate> {
  /**
   * Start in the paused state.
   *
   * @default false
   */
  readonly initiallyPaused?: boolean;
}

/**
 * Watch sources like Vue's `watch`, with pause and resume.
 *
 * Unlike the `pause()` built into Vue 3.5 watch handles, which delivers one
 * catch-up call on resume, changes made while paused are dropped: the
 * callback only ever sees changes that happened while active. The old value
 * passed after a resume is the value from the most recent change Vue
 * observed, whether or not it was delivered. The watcher follows the owning
 * reactive scope and is safe during server rendering.
 *
 * @example
 * ```ts
 * const { pause, resume } = watchPausable(form, persist, { deep: true });
 * pause();
 * form.name = "x"; // not persisted
 * resume();
 * ```
 *
 * @param source Ref, getter, reactive object, or tuple of those.
 * @param callback Watch callback, skipped while paused.
 * @param options Watch options plus the initial pause state.
 * @default options {}
 * @returns Pause, resume, and stop controls with the active flag.
 */
export function watchPausable<
  const Sources extends WatchSources,
  Immediate extends Readonly<boolean> = false,
>(
  source: Sources,
  callback: WatchHelperCallback<Sources, Immediate>,
  options?: WatchPausableOptions<Immediate>,
): PausableWatchHandle;
export function watchPausable(
  source: WatchSources,
  callback: WatchCallback<unknown, unknown>,
  options: WatchPausableOptions<boolean> = {},
): PausableWatchHandle {
  const { initiallyPaused = false, ...watchOptions } = options;
  const isActive = shallowRef(!initiallyPaused);

  const handle = watch(
    source,
    (value, oldValue, onCleanup) => {
      if (isActive.value) callback(value, oldValue, onCleanup);
    },
    watchOptions,
  );

  return {
    isActive,
    pause: () => {
      isActive.value = false;
    },
    resume: () => {
      isActive.value = true;
    },
    stop: () => {
      handle.stop();
    },
  };
}
