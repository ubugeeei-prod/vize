import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { usePerformanceObserver } from "./use-performance-observer.ts";
import type {
  PerformanceEntryLike,
  PerformanceObserveInit,
  PerformanceObserverEntryListLike,
  PerformanceObserverHost,
  PerformanceObserverLike,
} from "./use-performance-observer.ts";

type ObserverCallback = (
  list: PerformanceObserverEntryListLike,
  observer: PerformanceObserverLike,
) => void;

interface FakeObserverInstance extends PerformanceObserverLike {
  readonly observed: PerformanceObserveInit[];
  readonly queue: PerformanceEntryLike[];
  readonly connected: boolean;
  emit(entries: readonly PerformanceEntryLike[]): void;
}

function createObserverClass(supportedEntryTypes?: readonly string[]): {
  Observer: PerformanceObserverHost;
  instances: FakeObserverInstance[];
} {
  const instances: FakeObserverInstance[] = [];
  class FakeObserver implements FakeObserverInstance {
    static readonly supportedEntryTypes = supportedEntryTypes;
    readonly observed: PerformanceObserveInit[] = [];
    readonly queue: PerformanceEntryLike[] = [];
    connected = true;
    readonly callback: ObserverCallback;

    constructor(callback: ObserverCallback) {
      this.callback = callback;
      instances.push(this);
    }

    observe(options: PerformanceObserveInit): void {
      if (options.type === "broken") throw new TypeError("unsupported");
      this.observed.push(options);
    }

    disconnect(): void {
      this.connected = false;
    }

    takeRecords(): readonly PerformanceEntryLike[] {
      return this.queue.splice(0);
    }

    emit(entries: readonly PerformanceEntryLike[]): void {
      if (this.connected) this.callback({ getEntries: () => entries }, this);
    }
  }
  return { Observer: FakeObserver, instances };
}

const lcp = {
  name: "",
  entryType: "largest-contentful-paint",
  startTime: 1200,
  duration: 0,
  renderTime: 1200,
  loadTime: 1100,
  size: 5000,
  id: "hero",
  url: "",
} as const;

const mark = { name: "ready", entryType: "mark", startTime: 5, duration: 0, detail: null } as const;

void test("observes each supported type with buffered and threshold options", () => {
  const { Observer, instances } = createObserverClass(["event", "largest-contentful-paint"]);
  const observer = usePerformanceObserver(
    ["event", "largest-contentful-paint", "longtask"],
    () => undefined,
    { PerformanceObserver: Observer, buffered: true, durationThreshold: 40 },
  );

  assert.equal(observer.supported.value, true);
  assert.deepEqual(observer.observedTypes.value, ["event", "largest-contentful-paint"]);
  assert.equal(observer.isActive.value, true);
  assert.deepEqual(instances[0]?.observed, [
    { type: "event", buffered: true, durationThreshold: 40 },
    { type: "largest-contentful-paint", buffered: true },
  ]);
});

void test("delivers only entries of the requested types", () => {
  const { Observer, instances } = createObserverClass();
  const received: unknown[] = [];
  usePerformanceObserver(
    "largest-contentful-paint",
    (entries) => {
      received.push(...entries.map((entry) => entry.size));
    },
    { PerformanceObserver: Observer },
  );

  instances[0]?.emit([lcp, mark]);
  instances[0]?.emit([mark]);
  assert.deepEqual(received, [5000]);
});

void test("takeRecords returns typed pending entries", () => {
  const { Observer, instances } = createObserverClass();
  const observer = usePerformanceObserver("mark", () => undefined, {
    PerformanceObserver: Observer,
  });
  instances[0]?.queue.push(mark, lcp);
  assert.deepEqual(
    observer.takeRecords().map((entry) => entry.name),
    ["ready"],
  );
  assert.deepEqual(observer.takeRecords(), []);
});

void test("stop disconnects and start reconnects", () => {
  const { Observer, instances } = createObserverClass();
  const observer = usePerformanceObserver("mark", () => undefined, {
    PerformanceObserver: Observer,
  });
  observer.stop();
  assert.equal(instances[0]?.connected, false);
  assert.equal(observer.isActive.value, false);
  assert.deepEqual(observer.takeRecords(), []);
  observer.start();
  assert.equal(instances.length, 2);
  assert.equal(observer.isActive.value, true);
});

void test("does not start until requested when immediate is false", () => {
  const { Observer, instances } = createObserverClass();
  const observer = usePerformanceObserver("mark", () => undefined, {
    PerformanceObserver: Observer,
    immediate: false,
  });
  assert.equal(instances.length, 0);
  observer.start();
  assert.equal(instances.length, 1);
});

void test("follows reactive entry types", async () => {
  const { Observer, instances } = createObserverClass();
  const types = ref<"mark" | "measure">("mark");
  usePerformanceObserver(types, () => undefined, { PerformanceObserver: Observer });
  types.value = "measure";
  await nextTick();
  assert.equal(instances.length, 2);
  assert.equal(instances[0]?.connected, false);
  assert.deepEqual(instances[1]?.observed, [{ type: "measure" }]);
});

void test("reports unsupported types and observe failures without throwing", () => {
  const { Observer } = createObserverClass(["mark"]);
  const unsupported = usePerformanceObserver("longtask", () => undefined, {
    PerformanceObserver: Observer,
  });
  assert.equal(unsupported.supported.value, false);
  assert.equal(unsupported.isActive.value, false);

  const failing = createObserverClass();
  const broken = usePerformanceObserver(
    // Simulates a runtime that rejects a type it claimed to support.
    shallowRef<"mark">("broken" as "mark"),
    () => undefined,
    { PerformanceObserver: failing.Observer },
  );
  assert.ok(broken.error.value instanceof TypeError);
  assert.equal(broken.isActive.value, false);
  assert.equal(failing.instances[0]?.connected, false);

  const missing = usePerformanceObserver("mark", () => undefined, { PerformanceObserver: null });
  assert.equal(missing.supported.value, false);
});

void test("rejects an invalid duration threshold", () => {
  assert.throws(
    () => usePerformanceObserver("event", () => undefined, { durationThreshold: -1 }),
    /VIZE_COMPOSE_PERF_OBSERVER_INVALID_DURATION_THRESHOLD/,
  );
});

void test("disconnects with the scope", () => {
  const { Observer, instances } = createObserverClass();
  const scope = effectScope();
  scope.run(() =>
    usePerformanceObserver("mark", () => undefined, { PerformanceObserver: Observer }),
  );
  scope.stop();
  assert.equal(instances[0]?.connected, false);
});

void test("server rendering observes nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const observer = usePerformanceObserver("paint", () => undefined, { buffered: true });
    return {
      supported: observer.supported,
      isActive: observer.isActive,
      observedTypes: observer.observedTypes,
    };
  });
  assert.equal(state, '{"supported":false,"isActive":false,"observedTypes":[]}');
});
