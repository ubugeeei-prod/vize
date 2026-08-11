/** Options for {@link calculateRetryDelay}. */
export interface RetryDelayOptions {
  /**
   * Delay before the first retry, in milliseconds.
   *
   * @default 100
   */
  readonly initialDelayMs?: number;

  /**
   * Exponential multiplier applied for each subsequent retry.
   *
   * Values may be fractional but must be finite and at least one. The
   * calculated delay is rounded up so a retry never starts earlier than the
   * requested backoff.
   *
   * @default 2
   */
  readonly multiplier?: number;

  /**
   * Inclusive ceiling for the calculated delay, in milliseconds.
   *
   * The ceiling may be lower than {@link RetryDelayOptions.initialDelayMs};
   * in that case it also caps the first retry.
   *
   * @default 30000
   */
  readonly maximumDelayMs?: number;

  /**
   * Fraction of the capped delay eligible for downward jitter.
   *
   * `0` is deterministic exponential backoff, `0.5` samples from the upper
   * half of the range, and `1` applies full jitter from zero through the
   * capped delay. Jitter never exceeds the unjittered delay.
   *
   * @default 0
   */
  readonly jitterRatio?: number;

  /**
   * Entropy source returning a number in the half-open interval `[0, 1)`.
   * It is called exactly once when the selected jitter range contains more
   * than one integer millisecond, and is otherwise not read.
   *
   * @default Math.random
   */
  readonly random?: () => number;
}

/**
 * Calculate a bounded exponential-backoff delay for a one-based retry.
 *
 * The first retry is `retryAttempt = 1`. The result is always an integer from
 * zero through `2_147_483_647`, making it safe to pass to common host timer
 * APIs without implementation-specific clamping. The calculation is pure
 * unless jitter requires the supplied entropy source, and it performs no work
 * when the module is imported.
 *
 * Jitter samples uniformly from the inclusive integer range
 * `ceil(cappedDelay * (1 - jitterRatio))...cappedDelay`. Capping occurs before
 * jitter, so neither floating-point overflow nor entropy can exceed the
 * configured maximum.
 *
 * @param retryAttempt One-based retry number after a failed initial attempt.
 * @param options Backoff, cap, jitter, and entropy configuration.
 * @default options {}
 * @throws {RangeError} A tagged error when the retry number, delay options, or
 * entropy value is outside its documented range.
 * @throws {TypeError} A tagged error when `options` is not an object or
 * `random` is not callable.
 * @returns An integer delay in milliseconds.
 */
export function calculateRetryDelay(retryAttempt: number, options: RetryDelayOptions = {}): number {
  if (!Number.isSafeInteger(retryAttempt) || retryAttempt < 1) {
    throw new RangeError(
      `[VIZE_COMPOSE_RETRY_INVALID_ATTEMPT] retryAttempt must be a positive safe integer; received ${String(retryAttempt)}`,
    );
  }
  if (options === null || typeof options !== "object") {
    throw new TypeError(
      `[VIZE_COMPOSE_RETRY_INVALID_OPTIONS] options must be an object; received ${options === null ? "null" : typeof options}`,
    );
  }

  const initialDelayMs = options.initialDelayMs === undefined ? 100 : options.initialDelayMs;
  const multiplier = options.multiplier === undefined ? 2 : options.multiplier;
  const maximumDelayMs = options.maximumDelayMs === undefined ? 30_000 : options.maximumDelayMs;
  const jitterRatio = options.jitterRatio === undefined ? 0 : options.jitterRatio;
  if (
    !isPortableDelay(initialDelayMs) ||
    !isPortableDelay(maximumDelayMs) ||
    !Number.isFinite(multiplier) ||
    multiplier < 1 ||
    !Number.isFinite(jitterRatio) ||
    jitterRatio < 0 ||
    jitterRatio > 1
  ) {
    throw new RangeError(
      `[VIZE_COMPOSE_RETRY_INVALID_OPTIONS] initialDelayMs and maximumDelayMs must be integers from 0 through ${String(maximumPortableTimeoutMs)}, multiplier must be finite and at least 1, and jitterRatio must be from 0 through 1; received initialDelayMs=${String(initialDelayMs)}, maximumDelayMs=${String(maximumDelayMs)}, multiplier=${String(multiplier)}, jitterRatio=${String(jitterRatio)}`,
    );
  }
  if (initialDelayMs === 0 || maximumDelayMs === 0) return 0;

  const scaledDelay = initialDelayMs * multiplier ** (retryAttempt - 1);
  const cappedDelay = Math.min(maximumDelayMs, Math.ceil(scaledDelay));
  const minimumDelay = Math.ceil(cappedDelay * (1 - jitterRatio));
  const integerRange = cappedDelay - minimumDelay;
  if (integerRange === 0) return cappedDelay;

  const random = options.random === undefined ? Math.random : options.random;
  if (typeof random !== "function") {
    throw new TypeError(
      `[VIZE_COMPOSE_RETRY_INVALID_RANDOM] random must be a function; received ${typeof random}`,
    );
  }
  const sample = random();
  if (!Number.isFinite(sample) || sample < 0 || sample >= 1) {
    throw new RangeError(
      `[VIZE_COMPOSE_RETRY_INVALID_RANDOM] random must return a finite number from 0 up to but excluding 1; received ${String(sample)}`,
    );
  }

  return minimumDelay + Math.floor(sample * (integerRange + 1));
}

function isPortableDelay(value: number): boolean {
  return Number.isSafeInteger(value) && value >= 0 && value <= maximumPortableTimeoutMs;
}

const maximumPortableTimeoutMs = 2_147_483_647;
