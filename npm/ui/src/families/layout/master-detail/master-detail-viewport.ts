import { getCurrentScope, onMounted, onScopeDispose, shallowRef, toValue } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

const setupDiagnostic = "VIZE_UI_MASTER_DETAIL_SETUP";

/**
 * Track the viewport width hydration-safely: `ssrWidth` (or `null`) until
 * mount, then `window.innerWidth` on every resize until the scope stops.
 *
 * Kept local to the family (rather than reusing `useBreakpoint`) so the
 * master-detail and responsive subpaths stay independent bundles.
 *
 * @param ssrWidth Width assumed during server rendering and hydration.
 * @returns Readonly width ref.
 */
export function useMasterDetailViewport(
  ssrWidth: MaybeRefOrGetter<number | undefined>,
): Readonly<Ref<number | null>> {
  if (getCurrentScope() === undefined) {
    throw new Error(`${setupDiagnostic}: call inside component setup`);
  }
  const width = shallowRef<number | null>(toValue(ssrWidth) ?? null);
  const update = (): void => {
    width.value = window.innerWidth;
  };
  onMounted(() => {
    update();
    window.addEventListener("resize", update, { passive: true });
  });
  onScopeDispose(() => {
    if (typeof window !== "undefined") window.removeEventListener("resize", update);
  });
  return width;
}
