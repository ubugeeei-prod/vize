import { computed, readonly, shallowRef, toValue, unref, watch } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Fields shared by every performance entry consumed by {@link usePerformanceObserver}. */
export interface PerformanceEntryLike {
  /** Entry name (URL, mark name, paint name, event type, …). */
  readonly name: string;
  /** Entry type this entry was reported for. */
  readonly entryType: string;
  /** Start time relative to the time origin, in milliseconds. */
  readonly startTime: number;
  /** Duration in milliseconds (`0` for instantaneous entries). */
  readonly duration: number;
}

/** `largest-contentful-paint` entry. */
export interface LargestContentfulPaintEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "largest-contentful-paint";
  /** Render time, or `0` for cross-origin images without Timing-Allow-Origin. */
  readonly renderTime: number;
  /** Load time of the resource, or `0` for text. */
  readonly loadTime: number;
  /** Visible area of the element in square pixels. */
  readonly size: number;
  /** Element `id`, or an empty string. */
  readonly id: string;
  /** Image URL, or an empty string for text. */
  readonly url: string;
}

/** `layout-shift` entry (Layout Instability API). */
export interface LayoutShiftEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "layout-shift";
  /** Layout shift score. */
  readonly value: number;
  /** Whether a user input happened within the previous 500 ms. */
  readonly hadRecentInput: boolean;
  /** Time of the most recent input. */
  readonly lastInputTime: number;
}

/** Fields of Event Timing entries (`event` and `first-input`). */
export interface EventTimingFieldsLike extends PerformanceEntryLike {
  /** Time the event handlers started running. */
  readonly processingStart: number;
  /** Time the event handlers finished running. */
  readonly processingEnd: number;
  /** Whether the event was cancelable. */
  readonly cancelable: boolean;
  /** Interaction identifier; `0` when the event is not part of an interaction. */
  readonly interactionId?: number;
}

/** `event` entry (Event Timing API). */
export interface EventTimingEntryLike extends EventTimingFieldsLike {
  /** Entry type discriminant. */
  readonly entryType: "event";
}

/** `first-input` entry (Event Timing API). */
export interface FirstInputEntryLike extends EventTimingFieldsLike {
  /** Entry type discriminant. */
  readonly entryType: "first-input";
}

/** `paint` entry (`first-paint` or `first-contentful-paint`). */
export interface PaintEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "paint";
}

/** Fields shared by resource and navigation timing entries. */
export interface ResourceTimingFieldsLike extends PerformanceEntryLike {
  /** Initiator (`navigation`, `script`, `img`, `fetch`, …). */
  readonly initiatorType: string;
  /** Time the request started. */
  readonly requestStart: number;
  /** Time the first response byte arrived. */
  readonly responseStart: number;
  /** Time the last response byte arrived. */
  readonly responseEnd: number;
  /** Transferred size in bytes including headers. */
  readonly transferSize: number;
}

/** `resource` entry (Resource Timing API). */
export interface ResourceTimingEntryLike extends ResourceTimingFieldsLike {
  /** Entry type discriminant. */
  readonly entryType: "resource";
}

/** `navigation` entry (Navigation Timing API). */
export interface NavigationTimingEntryLike extends ResourceTimingFieldsLike {
  /** Entry type discriminant. */
  readonly entryType: "navigation";
  /** Navigation type (`navigate`, `reload`, `back_forward`, `prerender`). */
  readonly type: string;
  /** Time a prerendered page was activated; `0` or absent otherwise. */
  readonly activationStart?: number;
  /** End of the `DOMContentLoaded` handlers. */
  readonly domContentLoadedEventEnd: number;
  /** End of the `load` handlers. */
  readonly loadEventEnd: number;
}

/** `longtask` entry (Long Tasks API). */
export interface LongTaskEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "longtask";
}

/** `mark` entry (User Timing API). */
export interface MarkEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "mark";
  /** Detail passed to `performance.mark`. */
  readonly detail: unknown;
}

/** `measure` entry (User Timing API). */
export interface MeasureEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "measure";
  /** Detail passed to `performance.measure`. */
  readonly detail: unknown;
}

/** `element` entry (Element Timing API). */
export interface ElementTimingEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "element";
  /** Value of the `elementtiming` attribute. */
  readonly identifier: string;
  /** Render time. */
  readonly renderTime: number;
  /** Load time of the resource, or `0` for text. */
  readonly loadTime: number;
  /** Element `id`, or an empty string. */
  readonly id: string;
  /** Image URL, or an empty string for text. */
  readonly url: string;
}

/** `long-animation-frame` entry (Long Animation Frames API). */
export interface LongAnimationFrameEntryLike extends PerformanceEntryLike {
  /** Entry type discriminant. */
  readonly entryType: "long-animation-frame";
  /** Start of the rendering phase. */
  readonly renderStart: number;
  /** Start of style and layout. */
  readonly styleAndLayoutStart: number;
  /** Total blocking time of the frame in milliseconds. */
  readonly blockingDuration: number;
  /** Time of the first UI event handled in this frame, or `0`. */
  readonly firstUIEventTimestamp: number;
}

/** Entry interface delivered for each supported entry type. */
export interface PerformanceEntryTypeMap {
  /** Largest Contentful Paint candidates. */
  readonly "largest-contentful-paint": LargestContentfulPaintEntryLike;
  /** Layout shifts. */
  readonly "layout-shift": LayoutShiftEntryLike;
  /** Event Timing entries. */
  readonly event: EventTimingEntryLike;
  /** The first input of the page. */
  readonly "first-input": FirstInputEntryLike;
  /** First paint and first contentful paint. */
  readonly paint: PaintEntryLike;
  /** Document navigation timing. */
  readonly navigation: NavigationTimingEntryLike;
  /** Resource timing. */
  readonly resource: ResourceTimingEntryLike;
  /** Tasks longer than 50 ms. */
  readonly longtask: LongTaskEntryLike;
  /** User Timing marks. */
  readonly mark: MarkEntryLike;
  /** User Timing measures. */
  readonly measure: MeasureEntryLike;
  /** Element Timing entries. */
  readonly element: ElementTimingEntryLike;
  /** Long animation frames. */
  readonly "long-animation-frame": LongAnimationFrameEntryLike;
}

/** Entry type understood by {@link usePerformanceObserver}. */
export type PerformanceEntryType = keyof PerformanceEntryTypeMap;

/** Options passed to {@link PerformanceObserverLike.observe}. */
export interface PerformanceObserveInit {
  /** Single entry type to observe. */
  readonly type: string;
  /** Replay entries buffered before observation started. */
  readonly buffered?: boolean;
  /** Minimum duration of reported `event` entries. */
  readonly durationThreshold?: number;
}

/** Minimal entry list handed to observer callbacks. */
export interface PerformanceObserverEntryListLike {
  /** Every entry of this batch. */
  getEntries(): readonly PerformanceEntryLike[];
}

/** Minimal `PerformanceObserver` instance. */
export interface PerformanceObserverLike {
  /** Start observing one entry type. */
  observe(options: PerformanceObserveInit): void;
  /** Stop observing everything. */
  disconnect(): void;
  /** Return and clear pending entries. */
  takeRecords(): readonly PerformanceEntryLike[];
}

/** Minimal `PerformanceObserver` constructor. */
export interface PerformanceObserverHost {
  /** Create an observer. */
  new (
    callback: (list: PerformanceObserverEntryListLike, observer: PerformanceObserverLike) => void,
  ): PerformanceObserverLike;
  /** Entry types this runtime supports; every type is attempted when absent. */
  readonly supportedEntryTypes?: readonly string[] | undefined;
}

/** Options for {@link usePerformanceObserver}. */
export interface UsePerformanceObserverOptions {
  /**
   * `PerformanceObserver` constructor for alternate runtimes and tests. A ref
   * (not a getter) because the host itself is a constructor function.
   *
   * @default window.PerformanceObserver when it exists
   */
  readonly PerformanceObserver?: MaybeRef<PerformanceObserverHost | null | undefined>;

  /**
   * Replay entries recorded before observation started.
   *
   * @default false
   */
  readonly buffered?: boolean;

  /**
   * Minimum duration in milliseconds of reported `event` entries (the
   * platform rounds it and clamps it to at least 16).
   *
   * @default undefined (platform default of 104)
   */
  readonly durationThreshold?: number;

  /**
   * Start observing immediately.
   *
   * @default true
   */
  readonly immediate?: boolean;
}

/** Callback invoked with correctly typed entries. */
export type PerformanceObserverCallback<Type extends PerformanceEntryType> = (
  entries: readonly PerformanceEntryTypeMap[Type][],
  observer: PerformanceObserverLike,
) => void;

/** Reactive state and actions returned by {@link usePerformanceObserver}. */
export interface PerformanceObserverControls<Type extends PerformanceEntryType> {
  /** Whether an observer exists and supports at least one requested type. */
  readonly supported: ComputedRef<boolean>;

  /** Requested entry types the runtime supports. */
  readonly observedTypes: ComputedRef<readonly Type[]>;

  /** Whether an observer is currently connected. */
  readonly isActive: Readonly<ShallowRef<boolean>>;

  /** Most recent failure thrown by `observe`, cleared on the next start. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /** Start (or restart) observing. */
  readonly start: () => void;

  /** Disconnect the observer. Repeated calls are safe. */
  readonly stop: () => void;

  /**
   * Return and clear entries that are queued but not yet delivered.
   *
   * @returns Pending entries of the requested types.
   */
  readonly takeRecords: () => readonly PerformanceEntryTypeMap[Type][];
}

function isPerformanceObserverHost(candidate: unknown): candidate is PerformanceObserverHost {
  return typeof candidate === "function";
}

function browserPerformanceObserver(): PerformanceObserverHost | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, "PerformanceObserver");
  return isPerformanceObserverHost(candidate) ? candidate : undefined;
}

function isEntryOf<Type extends PerformanceEntryType>(
  entry: PerformanceEntryLike,
  types: readonly Type[],
): entry is PerformanceEntryTypeMap[Type] {
  return types.some((type) => type === entry.entryType);
}

/**
 * Observe performance entries with callbacks typed by entry type.
 *
 * Each supported type is observed with its own `observe({ type })` call so
 * `buffered` and `durationThreshold` apply. Types missing from
 * `PerformanceObserver.supportedEntryTypes` are skipped; `supported` is false
 * when none remain. The observer follows reactive `entryTypes` and is
 * disconnected when the owning reactive scope stops; outside a scope the
 * caller owns `stop()`.
 *
 * Server rendering: no observer is created and `supported` is false.
 *
 * @example
 * ```ts
 * usePerformanceObserver("largest-contentful-paint", (entries) => {
 *   console.log(entries.at(-1)?.renderTime);
 * }, { buffered: true });
 * ```
 *
 * @param entryTypes One entry type or a list of them.
 * @param callback Receives batches of typed entries.
 * @param options Constructor host and observation options.
 * @default options {}
 * @returns Observer state and actions.
 */
export function usePerformanceObserver<const Type extends PerformanceEntryType>(
  entryTypes: MaybeRefOrGetter<Type | readonly Type[]>,
  callback: PerformanceObserverCallback<Type>,
  options: UsePerformanceObserverOptions = {},
): PerformanceObserverControls<Type> {
  const { durationThreshold } = options;
  if (
    durationThreshold !== undefined &&
    (!Number.isFinite(durationThreshold) || durationThreshold < 0)
  ) {
    throw new RangeError(
      `[VIZE_COMPOSE_PERF_OBSERVER_INVALID_DURATION_THRESHOLD] durationThreshold must be a finite, non-negative number, received ${String(durationThreshold)}`,
    );
  }

  const isActive = shallowRef(false);
  const error = shallowRef<unknown>(undefined);
  let observer: PerformanceObserverLike | undefined;

  const resolveHost = (): PerformanceObserverHost | undefined =>
    options.PerformanceObserver === undefined
      ? browserPerformanceObserver()
      : (unref(options.PerformanceObserver) ?? undefined);

  const requestedTypes = (): readonly Type[] => {
    const value = toValue(entryTypes);
    return typeof value === "string" ? [value] : [...new Set(value)];
  };

  const observedTypes = computed((): readonly Type[] => {
    const host = resolveHost();
    if (!host) return [];
    const supportedTypes = host.supportedEntryTypes;
    const requested = requestedTypes();
    return supportedTypes ? requested.filter((type) => supportedTypes.includes(type)) : requested;
  });

  const disconnect = (): void => {
    observer?.disconnect();
    observer = undefined;
    isActive.value = false;
  };

  const connect = (): void => {
    disconnect();
    const host = resolveHost();
    const types = observedTypes.value;
    if (!host || types.length === 0) return;
    error.value = undefined;
    const next = new host((list, current) => {
      const entries = list.getEntries().filter((entry) => isEntryOf(entry, types));
      if (entries.length > 0) callback(entries, current);
    });
    let observing = 0;
    for (const type of types) {
      try {
        next.observe({
          type,
          ...(options.buffered === undefined ? {} : { buffered: options.buffered }),
          ...(type === "event" && durationThreshold !== undefined ? { durationThreshold } : {}),
        });
        observing += 1;
      } catch (cause) {
        error.value = cause;
      }
    }
    if (observing === 0) {
      next.disconnect();
      return;
    }
    observer = next;
    isActive.value = true;
  };

  let enabled = options.immediate ?? true;
  const stopWatch = watch(
    () => [resolveHost(), observedTypes.value.join(" ")],
    () => {
      if (enabled) connect();
    },
    { immediate: true, flush: "sync" },
  );

  const stop = (): void => {
    enabled = false;
    disconnect();
  };

  tryOnScopeDispose(() => {
    stopWatch();
    stop();
  });

  return {
    supported: computed(() => observedTypes.value.length > 0),
    observedTypes,
    isActive: readonly(isActive),
    error: readonly(error),
    start: () => {
      enabled = true;
      connect();
    },
    stop,
    takeRecords: () => {
      const types = observedTypes.value;
      return observer ? observer.takeRecords().filter((entry) => isEntryOf(entry, types)) : [];
    },
  };
}
