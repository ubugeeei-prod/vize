import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type {
  NavigationTimingEntryLike,
  PerformanceEntryLike,
  PerformanceObserveInit,
  PerformanceObserverEntryListLike,
  PerformanceObserverHost,
  PerformanceObserverLike,
} from "./use-performance-observer.ts";
import { WEB_VITALS_THRESHOLDS, rateMetric, useWebVitals } from "./use-web-vitals.ts";
import type { WebVitalMetric } from "./use-web-vitals.ts";

type ObserverCallback = (
  list: PerformanceObserverEntryListLike,
  observer: PerformanceObserverLike,
) => void;

function createPerformance(): {
  Observer: PerformanceObserverHost;
  emit: (entries: readonly PerformanceEntryLike[]) => void;
  queue: (entries: readonly PerformanceEntryLike[]) => void;
  connected: (type: string) => boolean;
} {
  const observers: FakeObserver[] = [];
  class FakeObserver implements PerformanceObserverLike {
    readonly types: string[] = [];
    readonly pending: PerformanceEntryLike[] = [];
    connected = true;
    readonly callback: ObserverCallback;

    constructor(callback: ObserverCallback) {
      this.callback = callback;
      observers.push(this);
    }

    observe(options: PerformanceObserveInit): void {
      this.types.push(options.type);
    }

    disconnect(): void {
      this.connected = false;
    }

    takeRecords(): readonly PerformanceEntryLike[] {
      return this.pending.splice(0);
    }
  }
  const matching = (entries: readonly PerformanceEntryLike[]) =>
    observers.flatMap((observer) => {
      const own = entries.filter((entry) => observer.types.includes(entry.entryType));
      return observer.connected && own.length > 0 ? [{ observer, own }] : [];
    });
  return {
    Observer: FakeObserver,
    emit: (entries) => {
      for (const { observer, own } of matching(entries)) {
        observer.callback({ getEntries: () => own }, observer);
      }
    },
    queue: (entries) => {
      for (const { observer, own } of matching(entries)) observer.pending.push(...own);
    },
    connected: (type) =>
      observers.some((observer) => observer.connected && observer.types.includes(type)),
  };
}

class FakeDocument extends EventTarget {
  visibilityState = "visible";

  hide(): void {
    this.visibilityState = "hidden";
    this.dispatchEvent(new Event("visibilitychange"));
  }
}

const entry = { name: "", duration: 0 };
const lcpEntry = (startTime: number) => ({
  ...entry,
  entryType: "largest-contentful-paint" as const,
  startTime,
  renderTime: startTime,
  loadTime: 0,
  size: 100,
  id: "",
  url: "",
});
const shift = (startTime: number, value: number, hadRecentInput = false) => ({
  ...entry,
  entryType: "layout-shift" as const,
  startTime,
  value,
  hadRecentInput,
  lastInputTime: 0,
});
const event = (interactionId: number, duration: number) => ({
  ...entry,
  entryType: "event" as const,
  startTime: 10,
  duration,
  processingStart: 11,
  processingEnd: 12,
  cancelable: true,
  interactionId,
});

function setup(reportAllChanges = false) {
  const performance = createPerformance();
  const document = new FakeDocument();
  const vitals = useWebVitals({
    PerformanceObserver: performance.Observer,
    document,
    reportAllChanges,
  });
  const reports: WebVitalMetric[] = [];
  vitals.onMetric((metric) => reports.push(metric));
  return { ...performance, document, vitals, reports };
}

void test("rates metrics with the standard thresholds", () => {
  assert.equal(rateMetric("LCP", 2500), "good");
  assert.equal(rateMetric("LCP", 2501), "needs-improvement");
  assert.equal(rateMetric("LCP", 4001), "poor");
  assert.equal(rateMetric("CLS", 0.1), "good");
  assert.equal(rateMetric("CLS", 0.2), "needs-improvement");
  assert.equal(rateMetric("INP", 600), "poor");
  assert.equal(rateMetric("TTFB", 900), "needs-improvement");
  assert.deepEqual(WEB_VITALS_THRESHOLDS.FCP, [1800, 3000]);
  assert.throws(() => rateMetric("FCP", Number.NaN), /VIZE_COMPOSE_WEB_VITALS_INVALID_VALUE/);
});

void test("reports FCP and TTFB as soon as they are measured", () => {
  const { vitals, emit, reports, connected } = setup();
  emit([{ ...entry, entryType: "paint", name: "first-paint", startTime: 50 }]);
  emit([{ ...entry, entryType: "paint", name: "first-contentful-paint", startTime: 900 }]);
  const navigation: NavigationTimingEntryLike = {
    ...entry,
    entryType: "navigation",
    startTime: 0,
    initiatorType: "navigation",
    requestStart: 50,
    responseStart: 300,
    responseEnd: 400,
    transferSize: 1,
    type: "navigate",
    activationStart: 100,
    domContentLoadedEventEnd: 0,
    loadEventEnd: 0,
  };
  emit([navigation]);
  assert.equal(vitals.supported.value, true);
  assert.equal(vitals.metrics.value.FCP?.value, 900);
  assert.equal(vitals.metrics.value.FCP?.rating, "good");
  assert.equal(vitals.metrics.value.TTFB?.value, 200);
  assert.deepEqual(
    reports.map((metric) => [metric.name, metric.delta, metric.entries]),
    [
      ["FCP", 900, 1],
      ["TTFB", 200, 1],
    ],
  );
  assert.equal(connected("paint"), false);
  assert.equal(connected("navigation"), false);
});

void test("finalizes LCP on the first input", () => {
  const { vitals, emit, queue, document, reports, connected } = setup();
  emit([lcpEntry(1000)]);
  emit([lcpEntry(2600)]);
  assert.equal(vitals.metrics.value.LCP?.value, 2600);
  assert.equal(reports.length, 0);

  queue([lcpEntry(3000)]);
  document.dispatchEvent(new Event("keydown"));
  assert.equal(vitals.metrics.value.LCP?.value, 3000);
  assert.equal(vitals.metrics.value.LCP?.rating, "needs-improvement");
  assert.deepEqual(
    reports.map((metric) => metric.name),
    ["LCP"],
  );
  assert.equal(connected("largest-contentful-paint"), false);
  emit([lcpEntry(5000)]);
  assert.equal(vitals.metrics.value.LCP?.value, 3000);
});

void test("computes CLS from the largest session window", () => {
  const { vitals, emit, document, reports } = setup();
  emit([shift(0, 0.05), shift(500, 0.05), shift(1200, 0.05)]);
  assert.ok(Math.abs((vitals.metrics.value.CLS?.value ?? 0) - 0.15) < 1e-9);
  assert.equal(vitals.metrics.value.CLS?.entries, 3);

  // A gap over one second starts a new, smaller window.
  emit([shift(3000, 0.02), shift(3100, 0.9, true)]);
  assert.ok(Math.abs((vitals.metrics.value.CLS?.value ?? 0) - 0.15) < 1e-9);

  // Windows are capped at five seconds.
  emit([
    shift(10_000, 0.05),
    shift(10_900, 0.05),
    shift(11_800, 0.05),
    shift(12_700, 0.05),
    shift(13_600, 0.05),
    shift(14_500, 0.05),
    shift(15_400, 0.2),
  ]);
  assert.ok(Math.abs((vitals.metrics.value.CLS?.value ?? 0) - 0.3) < 1e-9);
  assert.equal(vitals.metrics.value.CLS?.rating, "poor");

  document.hide();
  assert.deepEqual(
    reports.map((metric) => metric.name),
    ["CLS"],
  );
});

void test("reports CLS 0 on hidden when nothing shifted", () => {
  const { vitals, document, reports } = setup();
  document.hide();
  assert.equal(vitals.metrics.value.CLS?.value, 0);
  assert.deepEqual(
    reports.map((metric) => [metric.name, metric.value]),
    [["CLS", 0]],
  );
});

void test("computes INP from the longest interactions", () => {
  const { vitals, emit, queue, document, reports } = setup();
  emit([event(7, 80), event(7, 120), event(0, 900), event(14, 60)]);
  assert.equal(vitals.metrics.value.INP?.value, 120);
  assert.equal(vitals.metrics.value.INP?.entries, 2);

  // With 50+ interactions the single longest one is ignored.
  const many = Array.from({ length: 60 }, (_, index) => event(100 + index * 7, 50));
  emit([...many, event(1000, 700)]);
  assert.equal(vitals.metrics.value.INP?.value, 120);

  queue([event(2000, 300)]);
  document.hide();
  assert.equal(vitals.metrics.value.INP?.value, 300);
  const inp = reports.find((metric) => metric.name === "INP");
  assert.equal(inp?.value, 300);
  assert.equal(inp?.delta, 300);
});

void test("reportAllChanges reports every change with deltas", () => {
  const { emit, reports } = setup(true);
  emit([shift(0, 0.05)]);
  emit([shift(100, 0.03)]);
  assert.deepEqual(
    reports.map((metric) => [metric.name, Math.round(metric.delta * 100)]),
    [
      ["CLS", 5],
      ["CLS", 3],
    ],
  );
});

void test("ignores paints after the page was hidden", () => {
  const { vitals, emit, document } = setup();
  document.hide();
  emit([{ ...entry, entryType: "paint", name: "first-contentful-paint", startTime: 1e12 }]);
  emit([lcpEntry(1e12)]);
  assert.equal(vitals.metrics.value.FCP, undefined);
  assert.equal(vitals.metrics.value.LCP, undefined);
});

void test("stops observers and listeners with the scope", () => {
  const performance = createPerformance();
  const document = new FakeDocument();
  const scope = effectScope();
  const vitals = scope.run(() =>
    useWebVitals({ PerformanceObserver: performance.Observer, document }),
  );
  assert.ok(vitals);
  scope.stop();
  assert.equal(performance.connected("layout-shift"), false);
  document.hide();
  assert.deepEqual(vitals.metrics.value, {});
});

void test("reports unsupported without an observer", () => {
  const vitals = useWebVitals({ PerformanceObserver: null, document: null });
  assert.equal(vitals.supported.value, false);
  assert.deepEqual(vitals.metrics.value, {});
});

void test("server rendering measures nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const vitals = useWebVitals();
    return { supported: vitals.supported, metrics: vitals.metrics };
  });
  assert.equal(state, '{"supported":false,"metrics":{}}');
});
