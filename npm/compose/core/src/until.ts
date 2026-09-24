import { toValue, watch } from "vue";
import type { MaybeRefOrGetter, WatchHandle } from "vue";

import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Options shared by every {@link until} matcher. */
export interface UntilOptions {
  /**
   * Reject when the condition is not met within this many milliseconds.
   * `undefined` waits indefinitely (until the owning scope stops watching).
   *
   * @default undefined
   */
  readonly timeout?: number;

  /**
   * Abort the wait; the promise rejects with the signal's reason.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;

  /**
   * Watch flush timing. `"sync"` also works during server rendering.
   *
   * @default "sync"
   */
  readonly flush?: "pre" | "post" | "sync";

  /**
   * Watch nested properties of the value.
   *
   * @default false
   */
  readonly deep?: boolean;

  /**
   * Owns the timeout timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

/** Error code carried by the rejection when an {@link until} wait times out. */
export type UntilTimeoutCode = "VIZE_COMPOSE_UNTIL_TIMEOUT";

/** Rejection reason used when an {@link until} wait times out. */
export interface UntilTimeoutError extends Error {
  /** Stable machine-readable code. */
  readonly code: UntilTimeoutCode;
}

/** Values treated as falsy by `toBeTruthy`. */
type UntilFalsy = false | 0 | 0n | "" | null | undefined;

/** Matchers that wait for the source to satisfy a condition. */
export interface UntilMatchers<Value> {
  /**
   * Resolve once the value satisfies the type guard, narrowing the result.
   *
   * @returns The narrowed value.
   */
  toMatch<Narrowed extends Value>(
    condition: (value: Value) => value is Narrowed,
    options?: UntilOptions,
  ): Promise<Narrowed>;
  /**
   * Resolve once the value satisfies the predicate.
   *
   * @returns The matching value.
   */
  toMatch(condition: (value: Value) => boolean, options?: UntilOptions): Promise<Value>;

  /**
   * Resolve once the value is `Object.is`-equal to `expected` (itself
   * possibly reactive).
   *
   * @returns The expected value.
   */
  toBe<const Expected extends Value>(
    expected: MaybeRefOrGetter<Expected>,
    options?: UntilOptions,
  ): Promise<Expected>;

  /**
   * Resolve once the value is truthy.
   *
   * @returns The value without its falsy members.
   */
  toBeTruthy(options?: UntilOptions): Promise<Exclude<Value, UntilFalsy>>;

  /**
   * Resolve once the value is `null`.
   *
   * @returns `null`.
   */
  toBeNull(options?: UntilOptions): Promise<Extract<Value, null>>;

  /**
   * Resolve once the value is `undefined`.
   *
   * @returns `undefined`.
   */
  toBeUndefined(options?: UntilOptions): Promise<Extract<Value, undefined>>;

  /**
   * Resolve once the value is `NaN`.
   *
   * @returns `NaN`.
   */
  toBeNaN(options?: UntilOptions): Promise<Value>;

  /**
   * Resolve once an array or string value includes `element`.
   *
   * @returns The containing value.
   */
  toContain(
    element: Value extends readonly (infer Element)[]
      ? Element
      : Value extends string
        ? string
        : never,
    options?: UntilOptions,
  ): Promise<Value>;

  /**
   * Resolve after the value changed once.
   *
   * @returns The new value.
   */
  changed(options?: UntilOptions): Promise<Value>;

  /**
   * Resolve after the value changed `count` times.
   *
   * @returns The value after the last counted change.
   */
  changedTimes(count: number, options?: UntilOptions): Promise<Value>;
}

/** Negated matchers available through {@link UntilChain.not}. */
export interface UntilNegatedMatchers<Value> {
  /**
   * Resolve once the value is no longer `Object.is`-equal to `unexpected`.
   *
   * @returns The value.
   */
  toBe(unexpected: MaybeRefOrGetter<Value>, options?: UntilOptions): Promise<Value>;

  /**
   * Resolve once the value is falsy.
   *
   * @returns The falsy value.
   */
  toBeTruthy(options?: UntilOptions): Promise<Extract<Value, UntilFalsy>>;

  /**
   * Resolve once the value is not `null`.
   *
   * @returns The value without `null`.
   */
  toBeNull(options?: UntilOptions): Promise<Exclude<Value, null>>;

  /**
   * Resolve once the value is not `undefined`.
   *
   * @returns The value without `undefined`.
   */
  toBeUndefined(options?: UntilOptions): Promise<Exclude<Value, undefined>>;

  /**
   * Resolve once the value no longer satisfies the predicate.
   *
   * @returns The value.
   */
  toMatch(condition: (value: Value) => boolean, options?: UntilOptions): Promise<Value>;
}

/** Fluent wait builder returned by {@link until}. */
export interface UntilChain<Value> extends UntilMatchers<Value> {
  /** Inverted matchers. */
  readonly not: UntilNegatedMatchers<Value>;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

function createTimeoutError(timeout: number): UntilTimeoutError {
  return Object.assign(
    new Error(`[VIZE_COMPOSE_UNTIL_TIMEOUT] condition not met within ${timeout}ms`),
    {
      code: "VIZE_COMPOSE_UNTIL_TIMEOUT" as const,
    },
  );
}

/**
 * Await a reactive source reaching a condition.
 *
 * Every matcher resolves immediately when the current value already
 * satisfies it (except `changed`/`changedTimes`), and otherwise watches the
 * source (`flush: "sync"` by default, which also works inside server-side
 * `setup`). Result types are narrowed by the matcher: `toBeTruthy()` strips
 * falsy members, `not.toBeNull()` strips `null`, `toMatch(guard)` follows the
 * guard. Because a timeout rejects (it never resolves with an unmatched
 * value), the narrowing is always sound. Watchers and timers are released
 * as soon as the promise settles.
 *
 * @example
 * ```ts
 * const user = await until(currentUser).not.toBeNull({ timeout: 5_000 });
 * const ready = await until(status).toBe("ready");
 * ```
 *
 * @param source Ref or getter to observe.
 * @throws Returned promises reject with an {@link UntilTimeoutError} on
 * timeout, with the abort reason when the signal aborts, and with a
 * `RangeError` tagged `VIZE_COMPOSE_UNTIL_INVALID_OPTION` for a negative or
 * non-finite timeout or change count.
 * @returns The fluent matcher chain.
 */
export function until<Value>(source: MaybeRefOrGetter<Value>): UntilChain<Value> {
  const wait = <Result extends Value>(
    matches: (value: Value) => value is Result,
    options: UntilOptions,
    checkInitial = true,
  ): Promise<Result> =>
    new Promise<Result>((resolve, reject) => {
      const { timeout, signal, scheduler = defaultScheduler } = options;
      if (timeout !== undefined && (!Number.isFinite(timeout) || timeout < 0)) {
        reject(
          new RangeError(
            `[VIZE_COMPOSE_UNTIL_INVALID_OPTION] timeout must be finite and at least zero; received ${String(timeout)}`,
          ),
        );
        return;
      }
      if (signal?.aborted === true) {
        reject(signal.reason);
        return;
      }

      let handle: WatchHandle | undefined;
      let timer: unknown;
      let settled = false;
      const onAbort = (): void => {
        finish();
        reject(signal?.reason);
      };
      const finish = (): void => {
        settled = true;
        handle?.stop();
        if (timer !== undefined) scheduler.clearTimeout(timer);
        signal?.removeEventListener("abort", onAbort);
      };
      const accept = (value: Value): void => {
        if (settled || !matches(value)) return;
        finish();
        resolve(value);
      };

      if (checkInitial) accept(toValue(source));
      if (settled) return;

      handle = watch(() => toValue(source), accept, {
        flush: options.flush ?? "sync",
        deep: options.deep ?? false,
      });
      signal?.addEventListener("abort", onAbort, { once: true });
      if (timeout !== undefined) {
        timer = scheduler.setTimeout(() => {
          finish();
          reject(createTimeoutError(timeout));
        }, timeout);
      }
    });

  const any = (_value: Value): _value is Value => true;

  const changedTimes = (count: number, options: UntilOptions = {}): Promise<Value> => {
    if (!Number.isSafeInteger(count) || count < 1) {
      return Promise.reject(
        new RangeError(
          `[VIZE_COMPOSE_UNTIL_INVALID_OPTION] count must be a positive integer; received ${String(count)}`,
        ),
      );
    }
    let seen = 0;
    return wait(
      (value): value is Value => {
        seen += 1;
        return seen >= count && any(value);
      },
      options,
      false,
    );
  };

  const includes = (value: Value, element: unknown): boolean =>
    Array.isArray(value)
      ? value.includes(element)
      : typeof value === "string" && typeof element === "string" && value.includes(element);

  function toMatch<Narrowed extends Value>(
    condition: (value: Value) => value is Narrowed,
    options?: UntilOptions,
  ): Promise<Narrowed>;
  function toMatch(condition: (value: Value) => boolean, options?: UntilOptions): Promise<Value>;
  function toMatch(
    condition: (value: Value) => boolean,
    options: UntilOptions = {},
  ): Promise<Value> {
    return wait((value): value is Value => condition(value), options);
  }

  return {
    toMatch,
    toBe: <const Expected extends Value>(
      expected: MaybeRefOrGetter<Expected>,
      options: UntilOptions = {},
    ) => wait((value): value is Expected => Object.is(value, toValue(expected)), options),
    toBeTruthy: (options = {}) =>
      wait((value): value is Exclude<Value, UntilFalsy> => Boolean(value), options),
    toBeNull: (options = {}) =>
      wait((value): value is Extract<Value, null> => value === null, options),
    toBeUndefined: (options = {}) =>
      wait((value): value is Extract<Value, undefined> => value === undefined, options),
    toBeNaN: (options = {}) => wait((value): value is Value => Number.isNaN(value), options),
    toContain: (element, options = {}) =>
      wait((value): value is Value => includes(value, element), options),
    changed: (options = {}) => changedTimes(1, options),
    changedTimes,
    not: {
      toBe: (unexpected, options = {}) =>
        wait((value): value is Value => !Object.is(value, toValue(unexpected)), options),
      toBeTruthy: (options = {}) =>
        wait((value): value is Extract<Value, UntilFalsy> => !value, options),
      toBeNull: (options = {}) =>
        wait((value): value is Exclude<Value, null> => value !== null, options),
      toBeUndefined: (options = {}) =>
        wait((value): value is Exclude<Value, undefined> => value !== undefined, options),
      toMatch: (condition, options = {}) =>
        wait((value): value is Value => !condition(value), options),
    },
  };
}
