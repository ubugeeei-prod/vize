import { shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Document-like capability observed by {@link useActiveElement}. */
export interface ActiveElementHost extends EventTarget {
  /** Currently focused element. */
  readonly activeElement: Element | null;
}

/** Options for {@link useActiveElement}. */
export interface UseActiveElementOptions {
  /**
   * Descend into open shadow roots to report the innermost focused element.
   *
   * @default true
   */
  readonly deep?: boolean;

  /**
   * Reactive document capability for alternate runtimes and tests.
   *
   * @default globalThis.document when available
   */
  readonly host?: MaybeRefOrGetter<ActiveElementHost | null | undefined>;
}

/**
 * Track the focused element of a document.
 *
 * Listens for capturing `focus`/`blur` events on the document, so focus
 * changes anywhere (including inside shadow trees) update the ref.
 * Server renders expose `null`. Listeners are removed with the owning scope.
 *
 * @param options Shadow-DOM traversal and document capability.
 * @default options {}
 * @returns Readonly shallow ref of the active element.
 */
export function useActiveElement(
  options: UseActiveElementOptions = {},
): Readonly<ShallowRef<Element | null>> {
  const deep = options.deep ?? true;
  const active = shallowRef<Element | null>(null);

  const stop = watch(
    () => (options.host === undefined ? browserActiveElementHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      if (!host) {
        active.value = null;
        return;
      }
      const update = (): void => {
        active.value = readActive(host.activeElement, deep);
      };
      update();
      host.addEventListener("focus", update, { capture: true, passive: true });
      host.addEventListener("blur", update, { capture: true, passive: true });
      onCleanup(() => {
        host.removeEventListener("focus", update, { capture: true });
        host.removeEventListener("blur", update, { capture: true });
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return active;
}

function readActive(element: Element | null, deep: boolean): Element | null {
  let current = element;
  while (deep && current?.shadowRoot?.activeElement) current = current.shadowRoot.activeElement;
  return current;
}

function browserActiveElementHost(): ActiveElementHost | undefined {
  return typeof document !== "undefined" ? document : undefined;
}
