/**
 * Shared source and callback typing for the watch helpers
 * (`watchDebounced`, `watchThrottled`, `watchPausable`, `watchIgnorable`,
 * `watchOnce`, `whenever`).
 *
 * The helpers accept the same sources as Vue's `watch`: a ref, a getter, a
 * reactive object, or a tuple of those. This module declares types only and
 * contributes no runtime code.
 */
import type { WatchCallback, WatchSource } from "vue";

/** A single source accepted by Vue's `watch`: ref, getter, or reactive object. */
export type WatchSourceInput = WatchSource<unknown> | object;

/** Every source shape accepted by the watch helpers, including tuples. */
export type WatchSources = WatchSourceInput | readonly WatchSourceInput[];

/**
 * Value observed from a single source: the ref/getter value, or the reactive
 * object itself.
 */
export type WatchSourceValue<Source> =
  Source extends WatchSource<infer Value> ? Value : Source extends object ? Source : never;

/**
 * Value delivered to a watch callback for {@link WatchSources}: a tuple
 * source maps element-wise, any other source resolves through
 * {@link WatchSourceValue}.
 */
export type WatchValue<Sources> = Sources extends readonly unknown[]
  ? { -readonly [Index in keyof Sources]: WatchSourceValue<Sources[Index]> }
  : WatchSourceValue<Sources>;

/**
 * Previous value delivered to a watch callback: `undefined` is added only
 * when the watcher runs immediately (tuples get it per element, as in Vue).
 */
export type WatchOldValue<Sources, Immediate> = Immediate extends true
  ? Sources extends readonly unknown[]
    ? { -readonly [Index in keyof Sources]: WatchSourceValue<Sources[Index]> | undefined }
    : WatchSourceValue<Sources> | undefined
  : WatchValue<Sources>;

/** Callback typed from its sources and the `immediate` flag. */
export type WatchHelperCallback<Sources, Immediate = false> = WatchCallback<
  WatchValue<Sources>,
  WatchOldValue<Sources, Immediate>
>;

/** Cleanup registrar passed as the third argument of a watch callback. */
export type WatchCleanupRegistrar = Parameters<WatchCallback>[2];
