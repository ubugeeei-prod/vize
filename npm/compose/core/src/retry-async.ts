import { timeoutAbortSignal } from "./abort-signal.ts";
import { calculateRetryDelay, type RetryDelayOptions } from "./retry-delay.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Context supplied to every invocation of a retried operation. */
export interface RetryAttemptContext {
  /** One-based operation attempt, including the initial call. */
  readonly attempt: number;

  /** Shared cancellation signal for the complete retry execution. */
  readonly signal: AbortSignal;
}

/** Context supplied after an operation fails and before retry policy runs. */
export interface RetryFailureContext extends RetryAttemptContext {
  /** Exact value thrown or rejected by the failed operation. */
  readonly error: unknown;

  /** One-based retry that would follow this failure. */
  readonly retryAttempt: number;
}

/** Context supplied when an approved retry is about to wait. */
export interface RetryScheduledContext extends RetryFailureContext {
  /** One-based operation attempt that will run after the wait. */
  readonly nextAttempt: number;

  /** Calculated backoff delay in integer milliseconds. */
  readonly delayMs: number;
}

/** Options for {@link retryAsync}. */
export interface RetryAsyncOptions extends RetryDelayOptions {
  /**
   * Maximum retries after the initial operation attempt.
   *
   * @default 3
   */
  readonly maximumRetries?: number;

  /**
   * Cancels the active operation, policy hook, notification hook, or backoff
   * wait. The returned promise rejects with the signal's exact reason.
   *
   * @default a private non-aborting signal
   */
  readonly signal?: AbortSignal;

  /**
   * Deterministic or host-specific scheduler used for backoff waits.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Decide whether an operation failure is retryable. Returning `false`
   * rejects with the original operation error without calculating a delay.
   * The decision may be asynchronous and remains abortable.
   *
   * @default every failure is retryable while retries remain
   */
  readonly shouldRetry?: (context: RetryFailureContext) => boolean | PromiseLike<boolean>;

  /**
   * Observe an approved retry before its backoff wait begins. The hook may be
   * asynchronous and remains abortable; a hook failure is propagated exactly
   * and the next operation attempt is not started.
   *
   * @default undefined
   */
  readonly onRetry?: (context: RetryScheduledContext) => void | PromiseLike<void>;
}

/**
 * Execute an operation with bounded, abortable retries and deterministic delay policy.
 *
 * Synchronous throws and asynchronous rejections follow the same path. A
 * successful value is returned unchanged. Exhaustion rejects with the final
 * operation error, a negative retry decision rejects with the error that was
 * evaluated, and cancellation rejects with the signal's exact reason. These
 * values are deliberately not wrapped.
 *
 * Cancellation races every asynchronous stage, so callers are not forced to
 * wait for an operation or hook that ignores its signal. Late settlements are
 * still observed internally and cannot become unhandled rejections. Backoff
 * options and entropy are evaluated lazily only after a failure is approved
 * for retry.
 *
 * @typeParam Value Value produced by the operation.
 * @param operation Work to invoke with a one-based attempt and shared signal.
 * @param options Retry count, cancellation, policy, hooks, and delay options.
 * @default options {}
 * @throws {RangeError} A tagged error when `maximumRetries` or inherited delay
 * options are outside their documented ranges.
 * @throws {TypeError} A tagged error when `options` is not an object, a
 * callback is not callable, or a retry decision does not resolve to a boolean.
 * @returns The first successful operation value.
 */
export async function retryAsync<Value>(
  operation: (context: RetryAttemptContext) => Value | PromiseLike<Value>,
  options: RetryAsyncOptions = {},
): Promise<Value> {
  if (typeof operation !== "function") {
    throw new TypeError(
      `[VIZE_COMPOSE_RETRY_INVALID_OPERATION] operation must be a function; received ${typeof operation}`,
    );
  }
  if (options === null || typeof options !== "object") {
    throw new TypeError(
      `[VIZE_COMPOSE_RETRY_INVALID_OPTIONS] options must be an object; received ${options === null ? "null" : typeof options}`,
    );
  }
  const maximumRetries = options.maximumRetries === undefined ? 3 : options.maximumRetries;
  if (
    !Number.isSafeInteger(maximumRetries) ||
    maximumRetries < 0 ||
    maximumRetries >= Number.MAX_SAFE_INTEGER
  ) {
    throw new RangeError(
      `[VIZE_COMPOSE_RETRY_INVALID_MAXIMUM_RETRIES] maximumRetries must be an integer from 0 through ${String(Number.MAX_SAFE_INTEGER - 1)}; received ${String(maximumRetries)}`,
    );
  }
  const shouldRetryPolicy = options.shouldRetry;
  const retryObserver = options.onRetry;
  if (shouldRetryPolicy !== undefined && typeof shouldRetryPolicy !== "function") {
    throw new TypeError(
      `[VIZE_COMPOSE_RETRY_INVALID_CALLBACK] shouldRetry must be a function; received ${typeof shouldRetryPolicy}`,
    );
  }
  if (retryObserver !== undefined && typeof retryObserver !== "function") {
    throw new TypeError(
      `[VIZE_COMPOSE_RETRY_INVALID_CALLBACK] onRetry must be a function; received ${typeof retryObserver}`,
    );
  }

  const signal = options.signal === undefined ? new AbortController().signal : options.signal;
  let attempt = 1;
  while (true) {
    throwIfAborted(signal);
    const outcome = await capture(
      raceWithAbort(
        Promise.resolve().then(() => operation({ attempt, signal })),
        signal,
      ),
    );
    if (outcome.status === "success") return outcome.value;
    if (signal.aborted) throw signal.reason;
    if (attempt > maximumRetries) throw outcome.error;

    const failure: RetryFailureContext = {
      attempt,
      error: outcome.error,
      retryAttempt: attempt,
      signal,
    };
    if (shouldRetryPolicy !== undefined) {
      const shouldRetry = await raceWithAbort(
        Promise.resolve().then(() => shouldRetryPolicy(failure)),
        signal,
      );
      if (typeof shouldRetry !== "boolean") {
        throw new TypeError(
          `[VIZE_COMPOSE_RETRY_INVALID_DECISION] shouldRetry must resolve to a boolean; received ${typeof shouldRetry}`,
        );
      }
      if (!shouldRetry) throw outcome.error;
    }
    throwIfAborted(signal);

    const delayMs = calculateRetryDelay(attempt, options);
    const scheduled: RetryScheduledContext = {
      ...failure,
      delayMs,
      nextAttempt: attempt + 1,
    };
    if (retryObserver !== undefined) {
      await raceWithAbort(
        Promise.resolve().then(() => retryObserver(scheduled)),
        signal,
      );
    }
    await waitForRetry(delayMs, signal, options.scheduler);
    attempt += 1;
  }
}

type Captured<Value> =
  | { readonly status: "success"; readonly value: Value }
  | { readonly status: "error"; readonly error: unknown };

async function capture<Value>(promise: Promise<Value>): Promise<Captured<Value>> {
  try {
    return { status: "success", value: await promise };
  } catch (error) {
    return { status: "error", error };
  }
}

function raceWithAbort<Value>(promise: Promise<Value>, signal: AbortSignal): Promise<Value> {
  if (signal.aborted) return Promise.reject(signal.reason);

  return new Promise((resolve, reject) => {
    let listening = false;
    let settled = false;
    const cleanup = () => {
      if (!listening) return;
      listening = false;
      try {
        signal.removeEventListener("abort", onAbort);
      } catch {
        // A broken adapter cannot be allowed to strand the primary promise.
        // Native AbortSignal removal is non-throwing.
      }
    };
    const settle = (callback: () => void) => {
      if (settled) return;
      settled = true;
      cleanup();
      callback();
    };
    const onAbort = () => settle(() => reject(signal.reason));

    // Attach handlers before registering with a potentially non-standard
    // signal so the work remains observed even if registration throws.
    void promise.then(
      (value) => settle(() => resolve(value)),
      (error: unknown) => settle(() => reject(error)),
    );
    try {
      listening = true;
      signal.addEventListener("abort", onAbort, { once: true });
      if (settled) cleanup();
      else if (signal.aborted) onAbort();
    } catch (error) {
      settle(() => reject(error));
    }
  });
}

async function waitForRetry(
  delayMs: number,
  signal: AbortSignal,
  scheduler: TimeoutScheduler | undefined,
): Promise<void> {
  const elapsed = {};
  const delaySignal = timeoutAbortSignal(
    delayMs,
    scheduler === undefined ? { reason: elapsed, signal } : { reason: elapsed, scheduler, signal },
  );
  try {
    await raceWithAbort(new Promise<never>(() => undefined), delaySignal);
  } catch (reason) {
    if (reason === elapsed) return;
    throw reason;
  }
}

function throwIfAborted(signal: AbortSignal): void {
  if (signal.aborted) throw signal.reason;
}
