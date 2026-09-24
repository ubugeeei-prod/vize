import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Options for {@link usePageLeave}. */
export interface UsePageLeaveOptions {
  /**
   * Window-like target that receives `mouseout`/`mouseover`.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<EventTarget | null | undefined>;
}

/**
 * Detect when the mouse leaves the page (for exit-intent UI).
 *
 * A `mouseout` without a `relatedTarget` means the pointer left the
 * document; any `mouseover` means it came back. Server renders report
 * `false`; listeners are removed with the owning reactive scope.
 *
 * @param options Window capability.
 * @default options {}
 * @returns Readonly ref that is `true` while the pointer is outside the page.
 */
export function usePageLeave(options: UsePageLeaveOptions = {}): Readonly<Ref<boolean>> {
  const left = ref(false);
  const onOut = (event: Event): void => {
    const related = "relatedTarget" in event ? event.relatedTarget : null;
    left.value = related === null || related === undefined;
  };
  const onOver = (): void => {
    left.value = false;
  };

  const stop = watch(
    () => (options.host === undefined ? browserPageHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      if (!host) return;
      host.addEventListener("mouseout", onOut, { passive: true });
      host.addEventListener("mouseover", onOver, { passive: true });
      onCleanup(() => {
        host.removeEventListener("mouseout", onOut);
        host.removeEventListener("mouseover", onOver);
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return readonly(left);
}

function browserPageHost(): EventTarget | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
