import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope } from "vue";

import {
  createSizeObserver,
  createVisibilityObserver,
  useSizeObserver,
  type SizeObserverEntry,
} from "./measure.ts";

type ResizeCallback = (entries: readonly FakeResizeEntry[]) => void;

interface FakeResizeEntry {
  readonly target: Element;
  readonly borderBoxSize?: readonly { inlineSize: number; blockSize: number }[];
  readonly contentBoxSize?: readonly { inlineSize: number; blockSize: number }[];
  readonly contentRect: { width: number; height: number };
}

class FakeResizeObserver {
  static instances: FakeResizeObserver[] = [];
  observed: { target: Element; options: unknown }[] = [];
  unobserved: Element[] = [];
  disconnected = false;
  constructor(readonly callback: ResizeCallback) {
    FakeResizeObserver.instances.push(this);
  }
  observe(target: Element, options: unknown) {
    this.observed.push({ target, options });
  }
  unobserve(target: Element) {
    this.unobserved.push(target);
  }
  disconnect() {
    this.disconnected = true;
  }
}

class FakeIntersectionObserver {
  static instances: FakeIntersectionObserver[] = [];
  observed: Element[] = [];
  constructor(
    readonly callback: (entries: readonly unknown[]) => void,
    readonly init: unknown,
  ) {
    FakeIntersectionObserver.instances.push(this);
  }
  observe(target: Element) {
    this.observed.push(target);
  }
  unobserve(_target: Element) {}
  disconnect() {}
}

function withFakeObservers<Result>(run: () => Result): Result {
  const previousResize = globalThis.ResizeObserver;
  const previousIntersection = globalThis.IntersectionObserver;
  globalThis.ResizeObserver = FakeResizeObserver as unknown as typeof ResizeObserver;
  globalThis.IntersectionObserver =
    FakeIntersectionObserver as unknown as typeof IntersectionObserver;
  FakeResizeObserver.instances = [];
  FakeIntersectionObserver.instances = [];
  try {
    return run();
  } finally {
    globalThis.ResizeObserver = previousResize;
    globalThis.IntersectionObserver = previousIntersection;
  }
}

function element(): Element {
  return document.createElement("div");
}

test("reports batched size changes for observed elements", () => {
  withFakeObservers(() => {
    const batches: (readonly SizeObserverEntry[])[] = [];
    const observer = createSizeObserver({ onResize: (entries) => batches.push(entries) });
    const first = element();
    const second = element();
    const stranger = element();

    assert.equal(observer.isSupported, true);
    observer.observe(first);
    observer.observe(second);
    assert.equal(observer.observedCount.value, 2);

    const platform = FakeResizeObserver.instances[0];
    assert.ok(platform);
    platform.callback([
      {
        target: first,
        borderBoxSize: [{ inlineSize: 100, blockSize: 40 }],
        contentRect: { width: 90, height: 30 },
      },
      { target: stranger, contentRect: { width: 1, height: 1 } },
      {
        target: second,
        borderBoxSize: [{ inlineSize: 200, blockSize: 80 }],
        contentRect: { width: 190, height: 70 },
      },
    ]);

    assert.equal(batches.length, 1);
    assert.deepEqual(batches[0], [
      { target: first, width: 100, height: 40 },
      { target: second, width: 200, height: 80 },
    ]);
    observer.dispose();
    assert.equal(platform.disconnected, true);
  });
});

test("prefers the configured box size over the content rect", () => {
  withFakeObservers(() => {
    const sizes: SizeObserverEntry[] = [];
    const observer = createSizeObserver({
      box: "content-box",
      onResize: (entries) => sizes.push(...entries),
    });
    const host = element();
    observer.observe(host);
    const platform = FakeResizeObserver.instances[0];
    assert.deepEqual(platform?.observed[0]?.options, { box: "content-box" });

    platform?.callback([
      {
        target: host,
        contentBoxSize: [{ inlineSize: 90, blockSize: 30 }],
        contentRect: { width: 89, height: 29 },
      },
    ]);
    platform?.callback([{ target: host, contentRect: { width: 50, height: 25 } }]);

    assert.deepEqual(
      sizes.map(({ width, height }) => ({ width, height })),
      [
        { width: 90, height: 30 },
        { width: 50, height: 25 },
      ],
    );
    observer.dispose();
  });
});

test("keeps observation idempotent per element", () => {
  withFakeObservers(() => {
    const observer = createSizeObserver({ onResize: () => {} });
    const host = element();
    observer.observe(host);
    observer.observe(host);

    assert.equal(observer.observedCount.value, 1);
    assert.equal(FakeResizeObserver.instances[0]?.observed.length, 1);
    observer.dispose();
  });
});

test("stops reporting after unobserve and disconnect", () => {
  withFakeObservers(() => {
    const batches: unknown[] = [];
    const observer = createSizeObserver({ onResize: (entries) => batches.push(entries) });
    const first = element();
    const second = element();
    observer.observe(first);
    observer.observe(second);
    const platform = FakeResizeObserver.instances[0];

    observer.unobserve(first);
    assert.equal(observer.observedCount.value, 1);
    assert.deepEqual(platform?.unobserved, [first]);
    platform?.callback([{ target: first, contentRect: { width: 5, height: 5 } }]);
    assert.equal(batches.length, 0, "released targets are filtered out");

    observer.disconnect();
    assert.equal(observer.observedCount.value, 0);
    platform?.callback([{ target: second, contentRect: { width: 5, height: 5 } }]);
    assert.equal(batches.length, 0);
    observer.observe(second);
    assert.equal(observer.observedCount.value, 1, "disconnect keeps the controller usable");
    observer.dispose();
  });
});

test("no-ops without platform observer support", () => {
  const previousResize = globalThis.ResizeObserver;
  const previousIntersection = globalThis.IntersectionObserver;
  // @ts-expect-error simulate a server-like platform without observers.
  delete globalThis.ResizeObserver;
  // @ts-expect-error simulate a server-like platform without observers.
  delete globalThis.IntersectionObserver;
  try {
    const sizes = createSizeObserver({ onResize: () => {} });
    const visibility = createVisibilityObserver({ onVisibilityChange: () => {} });
    assert.equal(sizes.isSupported, false);
    assert.equal(visibility.isSupported, false);
    sizes.observe(element());
    visibility.observe(element());
    assert.equal(sizes.observedCount.value, 0);
    assert.equal(visibility.observedCount.value, 0);
    sizes.dispose();
    visibility.dispose();
  } finally {
    globalThis.ResizeObserver = previousResize;
    globalThis.IntersectionObserver = previousIntersection;
  }
});

test("reports batched visibility changes for observed elements", () => {
  withFakeObservers(() => {
    const batches: unknown[][] = [];
    const observer = createVisibilityObserver({
      rootMargin: "8px",
      threshold: 0.5,
      onVisibilityChange: (entries) => batches.push([...entries]),
    });
    const host = element();
    observer.observe(host);

    const platform = FakeIntersectionObserver.instances[0];
    assert.deepEqual(platform?.init, { root: null, rootMargin: "8px", threshold: 0.5 });
    platform?.callback([
      { target: host, isIntersecting: true, intersectionRatio: 0.75 },
      { target: element(), isIntersecting: true, intersectionRatio: 1 },
    ]);

    assert.deepEqual(batches, [[{ target: host, isIntersecting: true, intersectionRatio: 0.75 }]]);
    observer.dispose();
  });
});

test("validates options and observation targets", () => {
  withFakeObservers(() => {
    assert.throws(
      () => createSizeObserver({ onResize: 1 as unknown as () => void }),
      /VIZE_UI_MEASURE_OPTION: onResize must be a function/,
    );
    assert.throws(
      () => createSizeObserver({ box: "margin-box" as "border-box", onResize: () => {} }),
      /box must be border-box or content-box/,
    );
    assert.throws(
      () => createVisibilityObserver({ onVisibilityChange: null as unknown as () => void }),
      /onVisibilityChange must be a function/,
    );
    const observer = createSizeObserver({ onResize: () => {} });
    assert.throws(
      () => observer.observe(null as unknown as Element),
      /observation targets must be elements/,
    );
    observer.dispose();
  });
});

test("throws for observation after dispose", () => {
  withFakeObservers(() => {
    const observer = createSizeObserver({ onResize: () => {} });
    observer.dispose();
    observer.dispose();
    assert.throws(() => observer.observe(element()), /VIZE_UI_MEASURE_DISPOSED/);
    assert.throws(() => observer.disconnect(), /VIZE_UI_MEASURE_DISPOSED/);
  });
});

test("binds disposal to the owning effect scope", () => {
  withFakeObservers(() => {
    assert.throws(
      () => useSizeObserver({ onResize: () => {} }),
      /VIZE_UI_MEASURE_SETUP: use inside component setup or an active effect scope/,
    );

    const scope = effectScope();
    const observer = scope.run(() => useSizeObserver({ onResize: () => {} }));
    assert.ok(observer);
    observer.observe(element());
    scope.stop();
    assert.throws(() => observer.observe(element()), /VIZE_UI_MEASURE_DISPOSED/);
  });
});
