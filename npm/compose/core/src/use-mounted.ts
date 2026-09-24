import { getCurrentInstance, onMounted, shallowRef } from "vue";
import type { ShallowRef } from "vue";

/**
 * Whether the calling component has mounted.
 *
 * `false` during server rendering and during hydration, `true` from the
 * component's `onMounted` on: gate client-only rendering behind it to keep
 * hydration stable. Outside a component (no instance to mount) it stays
 * `false`. Uses only public lifecycle hooks, so it works in Vapor
 * components too.
 *
 * @example
 * ```ts
 * const mounted = useMounted();
 * // <ClientOnlyChart v-if="mounted" />
 * ```
 *
 * @returns Readonly mounted flag.
 */
export function useMounted(): Readonly<ShallowRef<boolean>> {
  const mounted = shallowRef(false);
  if (getCurrentInstance() !== null) {
    onMounted(() => {
      mounted.value = true;
    });
  }
  return mounted;
}
