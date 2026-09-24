import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { useDebounced } from "./use-debounced.ts";
import type { UseDebouncedOptions } from "./use-debounced.ts";

/**
 * Readonly ref that follows `source` after it stayed quiet for `waitMs`.
 *
 * Shorthand for `useDebounced(source, waitMs, options).debounced` when the
 * pending flag and cancel/flush controls are not needed. Inherits every rule
 * of {@link useDebounced}: synchronous mirroring on the server, reactive
 * wait, and scope-bound cleanup.
 *
 * @example
 * ```ts
 * const query = ref("");
 * const debouncedQuery = refDebounced(query, 300);
 * ```
 *
 * @param source Reactive source.
 * @param waitMs Reactive quiet period in milliseconds.
 * @param options Server and scheduler policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_DEBOUNCE_INVALID_WAIT` for invalid
 * waits (see {@link useDebounced}).
 * @returns The debounced readonly ref.
 */
export function refDebounced<Value>(
  source: MaybeRefOrGetter<Value>,
  waitMs: MaybeRefOrGetter<number>,
  options: UseDebouncedOptions = {},
): Readonly<ShallowRef<Value>> {
  return useDebounced(source, waitMs, options).debounced;
}
