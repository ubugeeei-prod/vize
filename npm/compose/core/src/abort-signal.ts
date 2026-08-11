/**
 * Create a signal that aborts when the first input signal aborts.
 *
 * The standard `AbortSignal.any()` implementation is used when available.
 * Older runtimes receive an equivalent listener-based implementation that
 * removes every retained listener as soon as the result aborts. The first
 * already-aborted input wins in iteration order, and its exact reason is
 * forwarded. An empty iterable returns a fresh signal that never aborts.
 *
 * This function reads runtime constructors only when called and is safe to
 * import during server rendering. Materializing the iterable happens before
 * listeners are attached, so an iterable that throws cannot leak a partial
 * subscription. If a non-standard signal throws while registering, listeners
 * already attached to earlier inputs are released before the error propagates.
 *
 * @param signals Abort signals to compose; consumed exactly once.
 * @returns A new first-abort-wins signal.
 */
export function anyAbortSignal(signals: Iterable<AbortSignal>): AbortSignal {
  const inputs = [...signals];
  const nativeAny = conformantNativeAbortSignalAny();
  if (nativeAny !== undefined) return nativeAny.call(AbortSignal, inputs);

  const controller = new AbortController();
  const listeners: Array<readonly [AbortSignal, () => void]> = [];
  const cleanup = () => {
    for (const [signal, listener] of listeners) {
      signal.removeEventListener("abort", listener);
    }
    listeners.length = 0;
  };
  const abortFrom = (signal: AbortSignal) => {
    if (controller.signal.aborted) return;
    cleanup();
    controller.abort(signal.reason);
  };

  for (const signal of inputs) {
    if (signal.aborted) {
      abortFrom(signal);
      break;
    }
    const listener = () => abortFrom(signal);
    listeners.push([signal, listener]);
    try {
      signal.addEventListener("abort", listener, { once: true });
    } catch (error) {
      cleanup();
      throw error;
    }

    // A host adapter may change state between the preflight read and listener
    // registration. The second read closes that race without changing native
    // EventTarget behavior.
    if (signal.aborted) {
      abortFrom(signal);
      break;
    }
  }

  return controller.signal;
}

let probedNativeAny: unknown;
let cachedNativeAny: typeof AbortSignal.any | undefined;

/** Resolve the current native implementation only after its first-reason contract passes. */
function conformantNativeAbortSignalAny(): typeof AbortSignal.any | undefined {
  let candidate: typeof AbortSignal.any | undefined;
  try {
    candidate = Reflect.get(AbortSignal, "any") as typeof AbortSignal.any | undefined;
  } catch {
    return undefined;
  }
  if (candidate === probedNativeAny) return cachedNativeAny;

  probedNativeAny = candidate;
  cachedNativeAny =
    typeof candidate === "function" && nativeAnyReasonIsStable(candidate) ? candidate : undefined;
  return cachedNativeAny;
}

/**
 * Detect runtimes whose combined reason changes when an earlier array member
 * aborts after the winner. The probe deliberately does not read the reason
 * until both inputs abort, catching lazy native implementations as well.
 */
function nativeAnyReasonIsStable(candidate: typeof AbortSignal.any): boolean {
  try {
    const late = new AbortController();
    const winner = new AbortController();
    const expected = {};
    const combined = candidate.call(AbortSignal, [late.signal, winner.signal]);
    winner.abort(expected);
    late.abort({});
    return combined.aborted && combined.reason === expected;
  } catch {
    return false;
  }
}

/** Options for {@link timeoutAbortSignal}. */
export interface TimeoutAbortSignalOptions {
  /**
   * Abort the returned signal early when this parent aborts.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;

  /**
   * Reason used when the timeout elapses. Parent cancellation always forwards
   * the parent's reason instead.
   *
   * @default DOMException("The operation timed out.", "TimeoutError")
   */
  readonly reason?: unknown;

  /**
   * Deterministic or host-specific single-shot timer implementation. Supplying
   * a scheduler selects the compatibility implementation.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

/** Options for {@link deadlineAbortSignal}. */
export interface DeadlineAbortSignalOptions extends TimeoutAbortSignalOptions {
  /**
   * Clock returning Unix epoch milliseconds.
   *
   * @default Date.now
   */
  readonly now?: () => number;
}

/**
 * Create a signal that aborts after a portable, non-negative delay.
 *
 * The native `AbortSignal.timeout()` implementation is used when available
 * and neither a custom scheduler nor a custom reason is supplied. Older
 * runtimes use the same owned-timer path as injected schedulers. Parent
 * cancellation is composed with first-reason-wins semantics. Compatibility
 * scheduling owns exactly one timer and removes its parent listener whenever
 * either source aborts. A zero delay remains asynchronous.
 *
 * The delay must be an integer from `0` through `2_147_483_647`; this common
 * signed 32-bit timer ceiling avoids host-specific clamping. The function
 * accesses timers and abort constructors only when called and is safe to
 * import during server rendering.
 *
 * @param delayMs Delay in milliseconds.
 * @param options Parent signal, timeout reason, and scheduler.
 * @default options {}
 * @throws {RangeError} Tagged `VIZE_COMPOSE_ABORT_TIMEOUT_INVALID_DELAY` when
 * the delay is fractional, negative, non-finite, or exceeds the portable
 * timer ceiling.
 * @returns A new timeout or parent-cancelled signal.
 */
export function timeoutAbortSignal(
  delayMs: number,
  options: TimeoutAbortSignalOptions = {},
): AbortSignal {
  assertPortableTimeout(delayMs);

  const parent = options.signal;
  if (parent?.aborted) {
    const controller = new AbortController();
    controller.abort(parent.reason);
    return controller.signal;
  }

  if (
    options.scheduler === undefined &&
    options.reason === undefined &&
    typeof AbortSignal.timeout === "function"
  ) {
    const timeout = AbortSignal.timeout(delayMs);
    return options.signal === undefined ? timeout : anyAbortSignal([options.signal, timeout]);
  }

  const controller = new AbortController();
  const scheduler = options.scheduler ?? defaultTimeoutScheduler;

  let handle: unknown;
  let timerPending = true;
  let parentListening = false;
  const stopTimer = () => {
    if (!timerPending) return;
    timerPending = false;
    scheduler.clearTimeout(handle);
    handle = undefined;
  };
  const stopParent = () => {
    if (!parentListening || parent === undefined) return;
    parentListening = false;
    parent.removeEventListener("abort", abortFromParent);
  };
  const abort = (reason: unknown) => {
    if (controller.signal.aborted) return;
    stopTimer();
    stopParent();
    controller.abort(reason);
  };
  const abortFromParent = () => abort(parent?.reason);
  const timeoutReason =
    options.reason === undefined
      ? new DOMException("The operation timed out.", "TimeoutError")
      : options.reason;

  handle = scheduler.setTimeout(() => {
    timerPending = false;
    handle = undefined;
    abort(timeoutReason);
  }, delayMs);
  if (controller.signal.aborted || parent === undefined) return controller.signal;

  try {
    // Assume registration may have succeeded before a non-standard adapter
    // throws. Removing an absent listener is harmless, while delaying this
    // ownership flag until after registration could leak a partial listener.
    parentListening = true;
    parent.addEventListener("abort", abortFromParent, { once: true });
    if (controller.signal.aborted) {
      parent.removeEventListener("abort", abortFromParent);
      parentListening = false;
      return controller.signal;
    }
    if (parent.aborted) abortFromParent();
  } catch (error) {
    stopParent();
    stopTimer();
    throw error;
  }
  return controller.signal;
}

/**
 * Create a timeout signal from an absolute Unix-epoch deadline.
 *
 * Fractional positive differences are rounded up so cancellation never occurs
 * before the requested deadline. Past deadlines become an asynchronous
 * zero-delay timeout. `Date` and numeric deadlines are both accepted.
 *
 * @param deadline Absolute deadline as a `Date` or Unix epoch milliseconds.
 * @param options Clock, parent signal, timeout reason, and scheduler.
 * @default options {}
 * @throws {RangeError} Tagged `VIZE_COMPOSE_ABORT_DEADLINE_INVALID` when the
 * deadline, current clock value, or their positive difference is non-finite
 * or cannot fit the portable timeout range.
 * @returns A new deadline or parent-cancelled signal.
 */
export function deadlineAbortSignal(
  deadline: Date | number,
  options: DeadlineAbortSignalOptions = {},
): AbortSignal {
  const deadlineMs = deadline instanceof Date ? deadline.getTime() : deadline;
  const nowMs = (options.now ?? Date.now)();
  const difference = deadlineMs - nowMs;
  if (
    !Number.isFinite(deadlineMs) ||
    !Number.isFinite(nowMs) ||
    !Number.isFinite(difference) ||
    difference > maximumPortableTimeoutMs
  ) {
    throw new RangeError(
      `[VIZE_COMPOSE_ABORT_DEADLINE_INVALID] deadline and now must produce a finite delay from 0 through ${String(maximumPortableTimeoutMs)} milliseconds; received deadline=${String(deadlineMs)}, now=${String(nowMs)}`,
    );
  }

  const { now: _now, ...timeoutOptions } = options;
  return timeoutAbortSignal(Math.ceil(Math.max(0, difference)), timeoutOptions);
}

function assertPortableTimeout(delayMs: number): void {
  if (!Number.isSafeInteger(delayMs) || delayMs < 0 || delayMs > maximumPortableTimeoutMs) {
    throw new RangeError(
      `[VIZE_COMPOSE_ABORT_TIMEOUT_INVALID_DELAY] delayMs must be an integer from 0 through ${String(maximumPortableTimeoutMs)}; received ${String(delayMs)}`,
    );
  }
}
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

const maximumPortableTimeoutMs = 2_147_483_647;

const defaultTimeoutScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};
