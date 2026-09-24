import { customRef, toValue } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Options for {@link refAutoReset}. */
export interface RefAutoResetOptions {
  /**
   * Applies the reset timer when no browser `window` is available. When
   * disabled, written values persist on the server (a render is a snapshot).
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Owns the reset timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

function resolveResetMs(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_REF_AUTO_RESET_INVALID_DELAY] afterMs must be finite and at least zero; received ${String(value)}`,
    );
  }
  return Math.trunc(value);
}

/**
 * Writable ref that falls back to a default value `afterMs` milliseconds
 * after each write.
 *
 * Typical for transient UI state such as "Copied!" labels. Every write
 * restarts the timer; the default (itself possibly reactive) is read when the
 * reset happens, and the ref reads the current default until the first write.
 * The pending reset is cancelled when the owning reactive scope stops.
 * Without a browser `window` (and without `runOnServer`) no timer starts.
 *
 * @example
 * ```ts
 * const message = refAutoReset("", 2_000);
 * message.value = "Copied!"; // back to "" after two seconds
 * ```
 *
 * @param defaultValue Reactive value restored after the delay.
 * @param afterMs Reactive delay in milliseconds, finite and at least zero.
 * @param options Server and scheduler policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_REF_AUTO_RESET_INVALID_DELAY` for
 * invalid delays, synchronously at creation and on every write.
 * @returns A writable ref.
 */
export function refAutoReset<Value>(
  defaultValue: MaybeRefOrGetter<Value>,
  afterMs: MaybeRefOrGetter<number>,
  options: RefAutoResetOptions = {},
): Ref<Value> {
  const scheduler = options.scheduler ?? defaultScheduler;
  resolveResetMs(toValue(afterMs));
  let handle: unknown;
  let written = false;
  let current = toValue(defaultValue);

  const cancel = (): void => {
    if (handle !== undefined) scheduler.clearTimeout(handle);
    handle = undefined;
  };

  tryOnScopeDispose(cancel);

  return customRef<Value>((track, trigger) => ({
    get: () => {
      track();
      return written ? current : toValue(defaultValue);
    },
    set: (next) => {
      const delay = resolveResetMs(toValue(afterMs));
      current = next;
      written = true;
      trigger();
      cancel();
      if (typeof window === "undefined" && !(options.runOnServer ?? false)) return;
      handle = scheduler.setTimeout(() => {
        handle = undefined;
        written = false;
        trigger();
      }, delay);
    },
  }));
}
