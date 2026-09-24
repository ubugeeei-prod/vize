import { shallowRef, watch } from "vue";
import type { ShallowRef } from "vue";

import { isElementNode, resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/**
 * Track the parent element of a reactive element target.
 *
 * Re-reads `parentElement` whenever the target resolves or changes (after
 * the DOM update). Server renders expose `null`.
 *
 * @param target Reactive element target.
 * @returns Readonly shallow ref of the parent element.
 */
export function useParentElement(target: MaybeElementTarget): Readonly<ShallowRef<Element | null>> {
  const parent = shallowRef<Element | null>(null);
  const stop = watch(
    () => resolveElement(target),
    (element) => {
      const candidate: unknown = element?.parentElement ?? null;
      parent.value = isElementNode(candidate) ? candidate : null;
    },
    { immediate: true, flush: "post" },
  );
  tryOnScopeDispose(() => stop.stop());
  return parent;
}
