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
  if (typeof AbortSignal.any === "function") return AbortSignal.any(inputs);

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
