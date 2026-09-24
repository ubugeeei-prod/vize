import { computed, getCurrentInstance, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { useMounted } from "./use-mounted.ts";

/**
 * Hydration-stable feature detection.
 *
 * Inside a component the result is `false` until the component mounts and
 * then reflects `check()`, so server and hydrating client render the same
 * fallback first. Outside a component it is evaluated right away when a
 * browser `window` exists and stays `false` otherwise. `check` may read
 * reactive state; the result recomputes when it changes.
 *
 * @example
 * ```ts
 * const isSupported = useSupported(() => "share" in navigator);
 * ```
 *
 * @param check Returns a truthy value when the feature is available.
 * @returns Computed support flag.
 */
export function useSupported(check: () => unknown): ComputedRef<boolean> {
  const ready =
    getCurrentInstance() === null ? shallowRef(typeof window !== "undefined") : useMounted();
  return computed(() => ready.value && Boolean(check()));
}
