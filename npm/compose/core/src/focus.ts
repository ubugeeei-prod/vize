import { computed, readonly, ref, watch } from "vue";
import type { Ref, WritableComputedRef } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Element that can receive programmatic focus (`HTMLElement`, `SVGElement`, `MathMLElement`). */
export type FocusableElement = Element & HTMLOrSVGElement;

/** Options for {@link useFocus}. */
export interface UseFocusOptions {
  /**
   * Focus the element as soon as it resolves.
   *
   * @default false
   */
  readonly initialValue?: boolean;

  /**
   * Only report focus when the user agent would show a focus ring
   * (`:focus-visible`).
   *
   * @default false
   */
  readonly focusVisible?: boolean;

  /**
   * Avoid scrolling the element into view when focusing programmatically.
   *
   * @default false
   */
  readonly preventScroll?: boolean;
}

/** Reactive focus state returned by {@link useFocus}. */
export interface FocusControls {
  /** Whether the element is focused. Assign `true`/`false` to focus/blur it. */
  readonly focused: WritableComputedRef<boolean>;
  /** Stop tracking. Idempotent. */
  readonly stop: () => void;
}

/**
 * Track and control focus of a single element.
 *
 * Listens for `focus`/`blur` on the resolved element and re-syncs when the
 * target changes. Assigning `focused` calls `focus()`/`blur()`. Server renders
 * always report `false`; `initialValue` is applied only after the element
 * resolves on the client, so hydration output is unaffected.
 *
 * @param target Reactive element target.
 * @param options Initial focus, focus-visible filtering, and scroll behavior.
 * @default options {}
 * @returns Writable focus ref and stop control.
 */
export function useFocus(target: MaybeElementTarget, options: UseFocusOptions = {}): FocusControls {
  const inner = ref(false);
  const element = computed(() => {
    const resolved = resolveElement(target);
    return resolved && isFocusable(resolved) ? resolved : null;
  });

  const read = (el: FocusableElement): boolean => {
    const active = el.ownerDocument.activeElement === el;
    if (!active || !options.focusVisible) return active;
    return safeMatches(el, ":focus-visible");
  };

  const stopWatch = watch(
    element,
    (el, _previous, onCleanup) => {
      if (!el) {
        inner.value = false;
        return;
      }
      const onFocus = (): void => {
        inner.value = read(el);
      };
      const onBlur = (): void => {
        inner.value = false;
      };
      el.addEventListener("focus", onFocus);
      el.addEventListener("blur", onBlur);
      inner.value = read(el);
      if (options.initialValue && !inner.value) {
        el.focus({ preventScroll: options.preventScroll ?? false });
      }
      onCleanup(() => {
        el.removeEventListener("focus", onFocus);
        el.removeEventListener("blur", onBlur);
      });
    },
    { immediate: true, flush: "post" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);

  const focused = computed({
    get: () => inner.value,
    set: (next: boolean) => {
      const el = element.value;
      if (!el) return;
      if (next) el.focus({ preventScroll: options.preventScroll ?? false });
      else el.blur();
    },
  });

  return { focused, stop };
}

/** Reactive state returned by {@link useFocusWithin}. */
export interface FocusWithinControls {
  /** Whether the element or any descendant has focus. */
  readonly focused: Readonly<Ref<boolean>>;
  /** Stop tracking. Idempotent. */
  readonly stop: () => void;
}

/**
 * Track whether focus is inside an element (the `:focus-within` state).
 *
 * Uses bubbling `focusin`/`focusout` and checks the `relatedTarget` so moving
 * focus between descendants never flickers to `false`. Server renders report
 * `false`.
 *
 * @param target Reactive element target.
 * @returns Readonly focus-within ref and stop control.
 */
export function useFocusWithin(target: MaybeElementTarget): FocusWithinControls {
  const focused = ref(false);
  const stopWatch = watch(
    () => resolveElement(target),
    (el, _previous, onCleanup) => {
      if (!el) {
        focused.value = false;
        return;
      }
      const onFocusIn = (): void => {
        focused.value = true;
      };
      const onFocusOut = (event: Event): void => {
        const next = "relatedTarget" in event ? event.relatedTarget : null;
        focused.value = isNode(next) && el.contains(next);
      };
      el.addEventListener("focusin", onFocusIn);
      el.addEventListener("focusout", onFocusOut);
      const active = el.ownerDocument.activeElement;
      focused.value = active !== null && el.contains(active);
      onCleanup(() => {
        el.removeEventListener("focusin", onFocusIn);
        el.removeEventListener("focusout", onFocusOut);
      });
    },
    { immediate: true, flush: "post" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);
  return { focused: readonly(focused), stop };
}

function isFocusable(element: Element): element is FocusableElement {
  return (
    "focus" in element &&
    typeof element.focus === "function" &&
    "blur" in element &&
    typeof element.blur === "function"
  );
}

function isNode(value: unknown): value is Node {
  return typeof value === "object" && value !== null && "nodeType" in value;
}

function safeMatches(element: Element, selector: string): boolean {
  try {
    return element.matches(selector);
  } catch {
    return true;
  }
}
