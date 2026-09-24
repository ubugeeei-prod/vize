import { shallowRef } from "vue";
import type { ComponentPublicInstance, ShallowRef } from "vue";

import { resolveElement } from "./element-target.ts";

/** Function ref accepted by `:ref` on elements and components. */
export type ElementRefSetter = (value: Element | ComponentPublicInstance | null) => void;

/** State returned by {@link useElementRef}. */
export interface ElementRefControls<TargetElement extends Element> {
  /** Resolved element (component roots are unwrapped); `null` while unmounted. */
  readonly element: Readonly<ShallowRef<TargetElement | null>>;
  /** Bind with `:ref="setRef"` on an element or component. */
  readonly setRef: ElementRefSetter;
}

/**
 * Capture a template element through a function ref.
 *
 * An instance-free alternative to "current element" helpers: it never reads
 * `getCurrentInstance()`, so it works in Vapor components, render functions,
 * and plain effect scopes. Component refs are unwrapped to their root
 * element, and an optional guard narrows the element type (elements failing
 * it resolve to `null`). The ref is `null` during server rendering.
 *
 * @returns Reactive element plus the function ref to bind.
 */
export function useElementRef(): ElementRefControls<Element>;
/**
 * Capture a template element narrowed by a type guard.
 *
 * @param guard Type guard; elements failing it resolve to `null`.
 * @returns Reactive narrowed element plus the function ref to bind.
 */
export function useElementRef<TargetElement extends Element>(
  guard: (element: Element) => element is TargetElement,
): ElementRefControls<TargetElement>;
export function useElementRef(guard?: (element: Element) => boolean): ElementRefControls<Element> {
  const element = shallowRef<Element | null>(null);
  const setRef: ElementRefSetter = (value) => {
    const resolved = resolveElement(value);
    element.value = resolved && (!guard || guard(resolved)) ? resolved : null;
  };
  return { element, setRef };
}
