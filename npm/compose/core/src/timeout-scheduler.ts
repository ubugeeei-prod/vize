/**
 * Single-shot timer host used by the debounced and throttled state
 * utilities.
 *
 * Implement this interface to integrate a deterministic test clock, a native
 * runtime timer, or an application-owned scheduler. Handles are opaque: the
 * utilities only hand them back to {@link TimeoutScheduler.clearTimeout}.
 * This module declares types only and contributes no runtime code.
 */
export interface TimeoutScheduler {
  /** Starts a single-shot callback and returns its opaque cancellation handle. */
  readonly setTimeout: (callback: () => void, delayMs: number) => unknown;

  /** Cancels a handle previously returned by {@link TimeoutScheduler.setTimeout}. */
  readonly clearTimeout: (handle: unknown) => void;
}
