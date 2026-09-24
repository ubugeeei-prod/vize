/** Compile-only assertions for the timing, date, and watch helper contracts. */

import { computed, reactive, ref, shallowRef } from "vue";
import type { ComputedRef, ShallowRef, WatchHandle } from "vue";

import { until } from "./until.ts";
import type { UntilChain } from "./until.ts";
import { useCountdown } from "./use-countdown.ts";
import { formatDate, useDateFormat } from "./use-date-format.ts";
import { useInterval, useIntervalFn } from "./use-interval.ts";
import type { IntervalControls, PausableControls } from "./use-interval.ts";
import { useNow, useTimestamp } from "./use-now.ts";
import type { NowControls, TimestampControls } from "./use-now.ts";
import { useRafFn } from "./use-raf-fn.ts";
import { formatTimeAgo, useTimeAgo } from "./use-time-ago.ts";
import type { TimeAgoControls, TimeAgoUnit } from "./use-time-ago.ts";
import { useTimeout, useTimeoutFn } from "./use-timeout.ts";
import type { TimeoutControls, TimeoutFnControls } from "./use-timeout.ts";
import { watchDebounced } from "./watch-debounced.ts";
import type { TimedWatchHandle } from "./watch-debounced.ts";
import { watchIgnorable } from "./watch-ignorable.ts";
import { watchOnce } from "./watch-once.ts";
import { watchPausable } from "./watch-pausable.ts";
import type { WatchValue } from "./watch-source.ts";
import { watchThrottled } from "./watch-throttled.ts";
import { whenever } from "./whenever.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

// Timers: the return type follows the literal `controls` flag.
const intervalFn = useIntervalFn(() => undefined, 100);
const counter = useInterval(100);
const intervalControls = useInterval(100, { controls: true });
type _IntervalFnIsPausable = Expect<Equal<typeof intervalFn, PausableControls>>;
type _IntervalCounterIsReadonly = Expect<Equal<typeof counter, Readonly<ShallowRef<number>>>>;
type _IntervalControlsFollowFlag = Expect<Equal<typeof intervalControls, IntervalControls>>;
// @ts-expect-error interval callbacks receive no arguments.
useIntervalFn((tick: number) => tick, 100);

const timeoutFn = useTimeoutFn((id: string, attempt: number) => [id, attempt], 100, {
  immediate: false,
});
type _TimeoutStartKeepsTheCallbackArguments = Expect<
  Equal<Parameters<typeof timeoutFn.start>, [id: string, attempt: number]>
>;
timeoutFn satisfies TimeoutFnControls<[id: string, attempt: number]>;
// @ts-expect-error a callback with parameters cannot start immediately without arguments.
useTimeoutFn((id: string) => id, 100);
// @ts-expect-error start keeps the argument types.
timeoutFn.start(1, 1);
const ready = useTimeout(100);
const timeoutControls = useTimeout(100, { controls: true });
type _TimeoutReadyIsComputed = Expect<Equal<typeof ready, ComputedRef<boolean>>>;
type _TimeoutControlsFollowFlag = Expect<Equal<typeof timeoutControls, TimeoutControls>>;

useRafFn(({ delta, timestamp }) => delta + timestamp, { fpsLimit: () => 30 });
// @ts-expect-error frame callbacks receive frame timing, not a bare timestamp.
useRafFn((timestamp: number) => timestamp);

const now = useNow();
type _NowIsComputedDate = Expect<Equal<typeof now, ComputedRef<Date>>>;
const nowControls = useNow({ controls: true, interval: "requestAnimationFrame" });
const timestamp = useTimestamp();
const timestampControls = useTimestamp({ controls: true });
type _NowControlsFollowFlag = Expect<Equal<typeof nowControls, NowControls>>;
type _TimestampIsReadonly = Expect<Equal<typeof timestamp, Readonly<ShallowRef<number>>>>;
type _TimestampControlsFollowFlag = Expect<Equal<typeof timestampControls, TimestampControls>>;
// @ts-expect-error only numeric periods or the animation-frame cadence are accepted.
useNow({ interval: "each-second" });

const countdown = useCountdown(10);
countdown.start(5);
type _CountdownRemainingIsReadonly = Expect<
  Equal<typeof countdown.remaining, Readonly<ShallowRef<number>>>
>;

// Dates: closed unit and option unions.
formatTimeAgo(new Date(), Date.now(), {
  units: ["minute", "hour"],
  numeric: "always",
}) satisfies string;
// @ts-expect-error units are a closed union.
formatTimeAgo(0, 0, { units: ["fortnight"] });
type _TimeAgoUnitIsClosed = Expect<
  Equal<TimeAgoUnit, "second" | "minute" | "hour" | "day" | "week" | "month" | "year">
>;
const agoControls = useTimeAgo(() => "2026-01-01", { controls: true, locale: ref("ja") });
type _TimeAgoControlsFollowFlag = Expect<Equal<typeof agoControls, TimeAgoControls>>;
const ago = useTimeAgo(0);
type _TimeAgoIsComputed = Expect<Equal<typeof ago, ComputedRef<string>>>;
formatDate(0, "YYYY-MM-DD") satisfies string;
formatDate(0, { dateStyle: "full" }, { timeZone: "Asia/Tokyo" }) satisfies string;
useDateFormat(
  shallowRef(new Date()),
  computed(() => "HH:mm"),
) satisfies ComputedRef<string>;
// @ts-expect-error Intl options are validated.
formatDate(0, { dateStyle: "tiny" });

// Watch helpers: sources and callbacks are typed like Vue's watch.
const count = ref(0);
const label = shallowRef<string | null>(null);
const state = reactive({ nested: { flag: true } });

watchDebounced(count, (value, oldValue) => {
  type _DebouncedValue = Expect<Equal<typeof value, number>>;
  type _DebouncedOldValue = Expect<Equal<typeof oldValue, number>>;
}) satisfies TimedWatchHandle;
watchDebounced(
  count,
  (_value, oldValue) => {
    type _ImmediateOldValueMayBeUndefined = Expect<Equal<typeof oldValue, number | undefined>>;
  },
  { immediate: true, debounce: 100 },
);
watchThrottled([count, label, () => state.nested.flag], ([first, second, third]) => {
  type _TupleValues = Expect<
    Equal<[typeof first, typeof second, typeof third], [number, string | null, boolean]>
  >;
});
watchPausable(state, (value) => {
  type _ReactiveObjectValue = Expect<Equal<typeof value, typeof state>>;
}).pause();
watchIgnorable(label, (value) => value?.toUpperCase()).ignoreUpdates(() => undefined);
watchOnce(
  () => count.value > 1,
  (value) => value satisfies boolean,
) satisfies WatchHandle;
// @ts-expect-error the callback value follows the source.
watchOnce(count, (value: string) => value);
// @ts-expect-error watchOnce owns the `once` option.
watchOnce(count, () => undefined, { once: false });

type _WatchValueMapsTuples = Expect<
  Equal<WatchValue<readonly [typeof count, () => string]>, [number, string]>
>;

whenever(label, (value) => {
  type _WheneverStripsFalsyMembers = Expect<Equal<typeof value, string>>;
});

// until: matcher results are narrowed soundly.
const chain = until(label);
chain satisfies UntilChain<string | null>;
type _NotNullStripsNull = Expect<Equal<Awaited<ReturnType<typeof chain.not.toBeNull>>, string>>;
type _TruthyStripsFalsy = Expect<Equal<Awaited<ReturnType<typeof chain.toBeTruthy>>, string>>;
type _ToBeNullIsNull = Expect<Equal<Awaited<ReturnType<typeof chain.toBeNull>>, null>>;
const literal = until(label).toBe("ready");
type _ToBeKeepsTheLiteral = Expect<Equal<Awaited<typeof literal>, "ready">>;
const guarded = until(count).toMatch((value): value is 1 | 2 => value === 1 || value === 2);
type _ToMatchFollowsTheGuard = Expect<Equal<Awaited<typeof guarded>, 1 | 2>>;
// @ts-expect-error toBe only accepts values of the source type.
void until(count).toBe("1");
// @ts-expect-error toContain requires an element of the array source.
void until(ref<number[]>([])).toContain("x");
