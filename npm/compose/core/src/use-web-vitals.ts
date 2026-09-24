import { computed, readonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import { usePerformanceObserver } from "./use-performance-observer.ts";
import type {
  EventTimingFieldsLike,
  LayoutShiftEntryLike,
  PerformanceObserverHost,
} from "./use-performance-observer.ts";

/** Core Web Vitals and supporting metrics computed by {@link useWebVitals}. */
export type WebVitalName = "LCP" | "CLS" | "INP" | "FCP" | "TTFB";

/** Rating bucket of a metric value. */
export type WebVitalRating = "good" | "needs-improvement" | "poor";

/**
 * Standard `[good, poor]` thresholds: values up to the first bound are
 * "good", values above the second are "poor". Times are milliseconds; CLS is
 * unitless.
 */
export const WEB_VITALS_THRESHOLDS = {
  LCP: [2500, 4000],
  CLS: [0.1, 0.25],
  INP: [200, 500],
  FCP: [1800, 3000],
  TTFB: [800, 1800],
} as const satisfies Readonly<Record<WebVitalName, readonly [number, number]>>;

/**
 * Rate a metric value with {@link WEB_VITALS_THRESHOLDS}.
 *
 * @param name Metric name.
 * @param value Metric value (milliseconds, or the unitless CLS score).
 * @returns The rating bucket.
 */
export function rateMetric(name: WebVitalName, value: number): WebVitalRating {
  if (Number.isNaN(value)) {
    throw new RangeError(`[VIZE_COMPOSE_WEB_VITALS_INVALID_VALUE] ${name} value must be a number`);
  }
  const [good, poor] = WEB_VITALS_THRESHOLDS[name];
  if (value > poor) return "poor";
  return value > good ? "needs-improvement" : "good";
}

/** Snapshot of one metric. */
export interface WebVitalMetric {
  /** Metric name. */
  readonly name: WebVitalName;
  /** Current value (milliseconds, or the unitless CLS score). */
  readonly value: number;
  /** Rating of `value`. */
  readonly rating: WebVitalRating;
  /** Change since the value last reported through `onMetric` (the full value before that). */
  readonly delta: number;
  /** Number of performance entries that make up `value`. */
  readonly entries: number;
  /** Identifier stable for this metric during the page load. */
  readonly id?: string;
}

/** Metrics measured so far; a metric is absent until it has a value. */
export type WebVitalsRecord = Readonly<Partial<Record<WebVitalName, WebVitalMetric>>>;

/** Document events used by {@link useWebVitals}. */
export type WebVitalsDocumentEvent = "visibilitychange" | "keydown" | "click";

/** Minimal document consumed by {@link useWebVitals}. */
export interface WebVitalsDocumentHost {
  /** Current visibility (`"visible"` or `"hidden"`). */
  readonly visibilityState: string;
  /** Subscribe to a document event. */
  addEventListener(
    type: WebVitalsDocumentEvent,
    listener: (event: { readonly timeStamp: number }) => void,
    options?: { readonly capture?: boolean },
  ): void;
  /** Unsubscribe from a document event. */
  removeEventListener(
    type: WebVitalsDocumentEvent,
    listener: (event: { readonly timeStamp: number }) => void,
    options?: { readonly capture?: boolean },
  ): void;
}

/** Options for {@link useWebVitals}. */
export interface UseWebVitalsOptions {
  /**
   * `PerformanceObserver` constructor for alternate runtimes and tests.
   *
   * @default window.PerformanceObserver when it exists
   */
  readonly PerformanceObserver?: MaybeRef<PerformanceObserverHost | null | undefined>;

  /**
   * Document whose visibility finalizes metrics and whose first input stops LCP.
   *
   * @default window.document when available
   */
  readonly document?: MaybeRefOrGetter<WebVitalsDocumentHost | null | undefined>;

  /**
   * Report every change through `onMetric` instead of only final values.
   *
   * @default false
   */
  readonly reportAllChanges?: boolean;

  /**
   * Minimum Event Timing duration considered for INP, in milliseconds.
   *
   * @default 40
   */
  readonly durationThreshold?: number;
}

/** Reactive state and actions returned by {@link useWebVitals}. */
export interface WebVitalsControls {
  /** Whether at least one metric can be observed. */
  readonly supported: ComputedRef<boolean>;

  /** Metrics measured so far, replaced on each change. */
  readonly metrics: Readonly<ShallowRef<WebVitalsRecord>>;

  /**
   * Subscribe to reported metrics. FCP and TTFB are reported once measured;
   * LCP once finalized (first input or hidden page); CLS and INP whenever
   * the page becomes hidden with a changed value. With `reportAllChanges`,
   * every change is reported.
   *
   * @param listener Receives each reported metric.
   * @returns Removes the listener.
   */
  readonly onMetric: (listener: (metric: WebVitalMetric) => void) => () => void;

  /** Stop every observer and listener. Repeated calls are safe. */
  readonly stop: () => void;
}

interface Interaction {
  readonly id: number;
  duration: number;
  entries: number;
}

const MAX_INTERACTIONS = 10;

function browserDocument(): WebVitalsDocumentHost | undefined {
  return typeof window === "undefined" ? undefined : window.document;
}

/**
 * Measure LCP, CLS, INP, FCP and TTFB locally, without dependencies.
 *
 * - LCP: last candidate before the first input or hidden page.
 * - CLS: largest session window (shifts < 1 s apart, window ≤ 5 s),
 *   ignoring shifts after recent input.
 * - INP: per-interaction maximum duration; the highest after ignoring one
 *   per 50 interactions (a p98 approximation over the 10 longest). The
 *   interaction count only includes interactions above `durationThreshold`.
 * - FCP: `first-contentful-paint` before the page was first hidden.
 * - TTFB: navigation `responseStart - activationStart`, clamped at 0.
 *
 * Values are not adjusted for prerender activation except TTFB. Observers
 * and listeners are released when the owning reactive scope stops; outside
 * a scope the caller owns `stop()`.
 *
 * Server rendering: nothing is observed, `metrics` is `{}` and `supported`
 * is false.
 *
 * @example
 * ```ts
 * const { metrics, onMetric } = useWebVitals();
 * onMetric((metric) => navigator.sendBeacon("/vitals", JSON.stringify(metric)));
 * ```
 *
 * @param options Hosts and reporting policy.
 * @default options {}
 * @returns Reactive metrics and a report hook.
 */
export function useWebVitals(options: UseWebVitalsOptions = {}): WebVitalsControls {
  const metrics = shallowRef<WebVitalsRecord>({});
  const reported = new Map<WebVitalName, number>();
  const listeners = new Set<(metric: WebVitalMetric) => void>();
  const reportAllChanges = options.reportAllChanges ?? false;
  const observerOptions =
    options.PerformanceObserver === undefined
      ? { buffered: true }
      : { buffered: true, PerformanceObserver: options.PerformanceObserver };
  let pageId: string | undefined;

  const resolveDocument = (): WebVitalsDocumentHost | undefined =>
    options.document === undefined ? browserDocument() : (toValue(options.document) ?? undefined);

  const report = (name: WebVitalName): void => {
    const metric = metrics.value[name];
    if (!metric || reported.get(name) === metric.value) return;
    reported.set(name, metric.value);
    for (const listener of listeners) listener(metric);
  };

  const set = (name: WebVitalName, value: number, entries: number): void => {
    pageId ??= `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
    const metric: WebVitalMetric = {
      name,
      value,
      rating: rateMetric(name, value),
      delta: value - (reported.get(name) ?? 0),
      entries,
      id: `${name}-${pageId}`,
    };
    metrics.value = { ...metrics.value, [name]: metric };
    if (reportAllChanges) report(name);
  };

  let firstHiddenTime = resolveDocument()?.visibilityState === "hidden" ? 0 : Infinity;

  // LCP
  let lcpFinal = false;
  const lcp = usePerformanceObserver(
    "largest-contentful-paint",
    (entries) => {
      if (lcpFinal) return;
      const last = entries.filter((entry) => entry.startTime < firstHiddenTime).at(-1);
      if (last) set("LCP", Math.max(last.startTime, 0), 1);
    },
    observerOptions,
  );

  // CLS
  let sessionValue = 0;
  let session: LayoutShiftEntryLike[] = [];
  const handleShifts = (entries: readonly LayoutShiftEntryLike[]): void => {
    for (const entry of entries) {
      if (entry.hadRecentInput) continue;
      const first = session[0];
      const last = session.at(-1);
      if (
        first &&
        last &&
        entry.startTime - last.startTime < 1000 &&
        entry.startTime - first.startTime < 5000
      ) {
        sessionValue += entry.value;
        session.push(entry);
      } else {
        sessionValue = entry.value;
        session = [entry];
      }
      if (sessionValue > (metrics.value.CLS?.value ?? 0)) set("CLS", sessionValue, session.length);
    }
  };
  const cls = usePerformanceObserver("layout-shift", handleShifts, observerOptions);

  // INP
  const longest: Interaction[] = [];
  let interactionCount = 0;
  let previousInteractionId = 0;
  const handleEvents = (entries: readonly EventTimingFieldsLike[]): void => {
    let changed = false;
    for (const entry of entries) {
      const id = entry.interactionId ?? 0;
      if (id === 0) continue;
      const existing = longest.find((interaction) => interaction.id === id);
      if (existing) {
        existing.entries += 1;
        existing.duration = Math.max(existing.duration, entry.duration);
      } else {
        if (id !== previousInteractionId) interactionCount += 1;
        longest.push({ id, duration: entry.duration, entries: 1 });
      }
      previousInteractionId = id;
      changed = true;
    }
    if (!changed) return;
    longest.sort((left, right) => right.duration - left.duration);
    longest.splice(MAX_INTERACTIONS);
    const candidate =
      longest[Math.min(longest.length - 1, Math.floor(interactionCount / 50))] ?? longest[0];
    if (candidate) set("INP", candidate.duration, candidate.entries);
  };
  const inp = usePerformanceObserver(["event", "first-input"], handleEvents, {
    ...observerOptions,
    durationThreshold: options.durationThreshold ?? 40,
  });

  // FCP
  const fcp = usePerformanceObserver(
    "paint",
    (entries) => {
      const entry = entries.find(
        (paint) => paint.name === "first-contentful-paint" && paint.startTime < firstHiddenTime,
      );
      if (!entry) return;
      fcp.stop();
      set("FCP", Math.max(entry.startTime, 0), 1);
      report("FCP");
    },
    observerOptions,
  );

  // TTFB
  const ttfb = usePerformanceObserver(
    "navigation",
    (entries) => {
      const entry = entries[0];
      if (!entry) return;
      ttfb.stop();
      set("TTFB", Math.max(entry.responseStart - (entry.activationStart ?? 0), 0), 1);
      report("TTFB");
    },
    observerOptions,
  );

  const finalizeLcp = (): void => {
    if (lcpFinal) return;
    const pending = lcp.takeRecords().filter((entry) => entry.startTime < firstHiddenTime);
    const last = pending.at(-1);
    if (last) set("LCP", Math.max(last.startTime, 0), 1);
    lcpFinal = true;
    lcp.stop();
    report("LCP");
  };

  const onHidden = (timeStamp: number): void => {
    firstHiddenTime = Math.min(firstHiddenTime, timeStamp);
    finalizeLcp();
    handleShifts(cls.takeRecords());
    handleEvents(inp.takeRecords());
    if (!metrics.value.CLS && cls.supported.value) set("CLS", 0, 0);
    report("CLS");
    report("INP");
  };

  const stopDocument = watch(
    resolveDocument,
    (document, _previous, onCleanup) => {
      if (!document) return;
      const onVisibility = (event: { readonly timeStamp: number }): void => {
        if (document.visibilityState === "hidden") onHidden(event.timeStamp);
      };
      const onInput = (): void => {
        finalizeLcp();
        removeInput();
      };
      const removeInput = (): void => {
        document.removeEventListener("keydown", onInput, { capture: true });
        document.removeEventListener("click", onInput, { capture: true });
      };
      document.addEventListener("visibilitychange", onVisibility);
      if (!lcpFinal) {
        document.addEventListener("keydown", onInput, { capture: true });
        document.addEventListener("click", onInput, { capture: true });
      }
      onCleanup(() => {
        document.removeEventListener("visibilitychange", onVisibility);
        removeInput();
      });
    },
    { immediate: true, flush: "sync" },
  );

  const stop = (): void => {
    stopDocument();
    for (const observer of [lcp, cls, inp, fcp, ttfb]) observer.stop();
  };

  tryOnScopeDispose(() => {
    stop();
    listeners.clear();
  });

  return {
    supported: computed(() =>
      [lcp, cls, inp, fcp, ttfb].some((observer) => observer.supported.value),
    ),
    metrics: readonly(metrics),
    onMetric: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    stop,
  };
}
