import { computed, toValue } from "vue";
import type { MaybeRefOrGetter, Ref, WritableComputedRef } from "vue";

/**
 * View a nullable ref through a default value.
 *
 * Reads return the source value, or the (possibly reactive) default while the
 * source holds `null` or `undefined`, so the read type drops those members.
 * Writes go straight to the source and accept its full type, including
 * `null`/`undefined` to fall back to the default again. Pure derived state:
 * SSR-safe and nothing to dispose.
 *
 * @example
 * ```ts
 * const raw = ref<string | undefined>();
 * const name = refDefault(raw, "Anonymous");
 * name.value; // "Anonymous"
 * name.value = "Ada"; // raw.value === "Ada"
 * ```
 *
 * @param source Writable nullable ref.
 * @param defaultValue Reactive fallback.
 * @returns A writable computed ref whose reads never yield `null`/`undefined`.
 */
export function refDefault<Value>(
  source: Ref<Value>,
  defaultValue: MaybeRefOrGetter<NonNullable<Value>>,
): WritableComputedRef<NonNullable<Value>, Value> {
  return computed<NonNullable<Value>, Value>({
    get: () => source.value ?? toValue(defaultValue),
    set: (value) => {
      source.value = value;
    },
  });
}
