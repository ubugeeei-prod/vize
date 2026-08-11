import assert from "node:assert/strict";
import { test } from "node:test";

import { anyAbortSignal } from "./abort-signal.ts";

class TrackingSignal extends EventTarget {
  aborted = false;
  onabort: ((this: AbortSignal, event: Event) => unknown) | null = null;
  reason: unknown;
  readonly abortListeners = new Set<EventListenerOrEventListenerObject>();

  override addEventListener(
    type: string,
    callback: EventListenerOrEventListenerObject | null,
    options?: AddEventListenerOptions | boolean,
  ): void {
    super.addEventListener(type, callback, options);
    if (type === "abort" && callback !== null) this.abortListeners.add(callback);
  }

  override removeEventListener(
    type: string,
    callback: EventListenerOrEventListenerObject | null,
    options?: EventListenerOptions | boolean,
  ): void {
    super.removeEventListener(type, callback, options);
    if (type === "abort" && callback !== null) this.abortListeners.delete(callback);
  }

  throwIfAborted(): void {
    if (this.aborted) throw this.reason;
  }

  abort(reason: unknown): void {
    if (this.aborted) return;
    this.aborted = true;
    this.reason = reason;
    this.dispatchEvent(new Event("abort"));
  }

  asAbortSignal(): AbortSignal {
    return this as unknown as AbortSignal;
  }
}

class RejectingSignal extends TrackingSignal {
  override addEventListener(
    type: string,
    callback: EventListenerOrEventListenerObject | null,
    options?: AddEventListenerOptions | boolean,
  ): void {
    super.addEventListener(type, callback, options);
    throw new Error("registration rejected");
  }
}

function withoutNativeAny<Value>(callback: () => Value): Value {
  const descriptor = Object.getOwnPropertyDescriptor(AbortSignal, "any");
  Object.defineProperty(AbortSignal, "any", {
    configurable: true,
    value: undefined,
    writable: true,
  });
  try {
    return callback();
  } finally {
    if (descriptor === undefined) delete (AbortSignal as { any?: unknown }).any;
    else Object.defineProperty(AbortSignal, "any", descriptor);
  }
}

function withNativeAny<Value>(
  replacement: (signals: AbortSignal[]) => AbortSignal,
  callback: () => Value,
): Value {
  const descriptor = Object.getOwnPropertyDescriptor(AbortSignal, "any");
  Object.defineProperty(AbortSignal, "any", {
    configurable: true,
    value: replacement,
    writable: true,
  });
  try {
    return callback();
  } finally {
    if (descriptor === undefined) delete (AbortSignal as { any?: unknown }).any;
    else Object.defineProperty(AbortSignal, "any", descriptor);
  }
}

void test("returns a distinct, pending signal for empty input", () => {
  const signal = anyAbortSignal([]);

  assert.equal(signal.aborted, false);
  assert.notEqual(signal, anyAbortSignal([]));
});

void test("forwards the first abort reason and ignores later inputs", () => {
  const first = new AbortController();
  const second = new AbortController();
  const signal = anyAbortSignal([first.signal, second.signal]);
  const firstReason = { code: "first" };

  second.abort(firstReason);
  first.abort(new Error("too late"));

  assert.equal(signal.aborted, true);
  assert.equal(signal.reason, firstReason);
});

void test("falls back when a lazy native implementation changes the winning reason", () => {
  let nativeCalls = 0;
  withNativeAny(
    (signals) => {
      nativeCalls += 1;
      return {
        get aborted() {
          return signals.some((signal) => signal.aborted);
        },
        get reason() {
          return signals.find((signal) => signal.aborted)?.reason;
        },
      } as AbortSignal;
    },
    () => {
      const late = new AbortController();
      const winner = new AbortController();
      const expected = { code: "winner" };
      const signal = anyAbortSignal([late.signal, winner.signal]);

      winner.abort(expected);
      late.abort(new Error("too late"));

      assert.equal(signal.reason, expected);
      assert.equal(nativeCalls, 1, "a rejected native candidate is used only for its probe");
    },
  );
});

void test("caches and delegates a conformant native implementation by identity", () => {
  let nativeCalls = 0;
  withNativeAny(
    (signals) => {
      nativeCalls += 1;
      const controller = new AbortController();
      for (const signal of signals) {
        const forward = () => controller.abort(signal.reason);
        if (signal.aborted) {
          forward();
          break;
        }
        signal.addEventListener("abort", forward, { once: true });
      }
      return controller.signal;
    },
    () => {
      const first = new AbortController();
      const expected = { code: "native" };
      const firstCombined = anyAbortSignal([first.signal]);
      first.abort(expected);
      assert.equal(firstCombined.reason, expected);
      assert.equal(nativeCalls, 2, "the first call performs one probe and one delegation");

      const second = new AbortController();
      const secondCombined = anyAbortSignal([second.signal]);
      second.abort(expected);
      assert.equal(secondCombined.reason, expected);
      assert.equal(nativeCalls, 3, "the same function identity is not probed twice");
    },
  );
});

void test("falls back when a native accessor cannot be read", () => {
  const descriptor = Object.getOwnPropertyDescriptor(AbortSignal, "any");
  let reads = 0;
  Object.defineProperty(AbortSignal, "any", {
    configurable: true,
    get() {
      reads += 1;
      throw new Error("native accessor unavailable");
    },
  });
  try {
    const controller = new AbortController();
    const expected = { code: "fallback" };
    const signal = anyAbortSignal([controller.signal]);
    controller.abort(expected);

    assert.equal(signal.reason, expected);
    assert.equal(reads, 1);
  } finally {
    if (descriptor === undefined) delete (AbortSignal as { any?: unknown }).any;
    else Object.defineProperty(AbortSignal, "any", descriptor);
  }
});

void test("uses iteration order when multiple inputs are already aborted", () => {
  const first = new AbortController();
  const second = new AbortController();
  first.abort("first");
  second.abort("second");

  assert.equal(anyAbortSignal([first.signal, second.signal]).reason, "first");
  assert.equal(anyAbortSignal([second.signal, first.signal]).reason, "second");
});

void test("fallback removes listeners from every input after abort", () => {
  withoutNativeAny(() => {
    const first = new TrackingSignal();
    const second = new TrackingSignal();
    const signal = anyAbortSignal([first.asAbortSignal(), second.asAbortSignal()]);

    assert.equal(first.abortListeners.size, 1);
    assert.equal(second.abortListeners.size, 1);
    first.abort("cancelled");

    assert.equal(signal.aborted, true);
    assert.equal(signal.reason, "cancelled");
    assert.equal(first.abortListeners.size, 0);
    assert.equal(second.abortListeners.size, 0);
    second.abort("too late");
    assert.equal(signal.reason, "cancelled");
  });
});

void test("fallback resolves already-aborted inputs without retaining listeners", () => {
  withoutNativeAny(() => {
    const pending = new TrackingSignal();
    const aborted = new TrackingSignal();
    aborted.abort({ source: "adapter" });

    const signal = anyAbortSignal([pending.asAbortSignal(), aborted.asAbortSignal()]);

    assert.equal(signal.reason, aborted.reason);
    assert.equal(pending.abortListeners.size, 0);
    assert.equal(aborted.abortListeners.size, 0);
  });
});

void test("materializes a failing iterable before fallback listener registration", () => {
  withoutNativeAny(() => {
    const input = new TrackingSignal();
    function* failingSignals(): Iterable<AbortSignal> {
      yield input.asAbortSignal();
      throw new Error("iteration failed");
    }

    assert.throws(() => anyAbortSignal(failingSignals()), /iteration failed/);
    assert.equal(input.abortListeners.size, 0);
  });
});

void test("fallback releases partial subscriptions when registration throws", () => {
  withoutNativeAny(() => {
    const attached = new TrackingSignal();
    const rejecting = new RejectingSignal();

    assert.throws(
      () => anyAbortSignal([attached.asAbortSignal(), rejecting.asAbortSignal()]),
      /registration rejected/,
    );
    assert.equal(attached.abortListeners.size, 0);
    assert.equal(rejecting.abortListeners.size, 0);
  });
});
