import { onScopeDispose, onWatcherCleanup, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Scroll direction driven by one scroll button. */
export type SelectScrollDirection = "down" | "up";

/** Options for {@link useSelectScrollButton}. */
export interface SelectScrollButtonOptions {
  readonly direction: MaybeRefOrGetter<SelectScrollDirection>;
  readonly viewport: MaybeRefOrGetter<HTMLElement | null>;
  readonly step: MaybeRefOrGetter<number>;
  readonly interval: MaybeRefOrGetter<number>;
}

/** Controller returned by {@link useSelectScrollButton}. */
export interface SelectScrollButtonController {
  /** Whether the viewport can scroll further in this direction. */
  readonly visible: Readonly<ShallowRef<boolean>>;

  /** Start repeated scrolling until {@link stop} or the edge is reached. */
  readonly start: () => void;

  /** Stop repeated scrolling. */
  readonly stop: () => void;

  /** Scroll by one step immediately. */
  readonly scrollOnce: () => void;

  /** Re-measure whether scrolling is possible. */
  readonly update: () => void;
}

/** Whether `element` can scroll further in `direction`. */
export function canSelectViewportScroll(
  element: Pick<HTMLElement, "clientHeight" | "scrollHeight" | "scrollTop">,
  direction: SelectScrollDirection,
): boolean {
  if (direction === "up") return element.scrollTop > 0;
  return Math.ceil(element.scrollTop + element.clientHeight) < element.scrollHeight;
}

/**
 * Hover-to-scroll behavior for Select scroll buttons.
 *
 * Measurement starts after mount, so server and hydration renders agree on a
 * hidden button; listeners and timers are released with the owning scope.
 */
export function useSelectScrollButton(
  options: SelectScrollButtonOptions,
): SelectScrollButtonController {
  const visible = shallowRef(false);
  let timer: ReturnType<typeof setInterval> | null = null;

  function update(): void {
    const element = toValue(options.viewport);
    visible.value =
      element !== null && canSelectViewportScroll(element, toValue(options.direction));
  }

  function stop(): void {
    if (timer !== null) clearInterval(timer);
    timer = null;
  }

  function scrollOnce(): void {
    const element = toValue(options.viewport);
    if (element === null) return;
    const delta = toValue(options.step) * (toValue(options.direction) === "up" ? -1 : 1);
    element.scrollTop = Math.max(0, element.scrollTop + delta);
    update();
    if (!visible.value) stop();
  }

  function start(): void {
    stop();
    scrollOnce();
    timer = setInterval(scrollOnce, Math.max(1, toValue(options.interval)));
  }

  watch(
    () => toValue(options.viewport),
    (element) => {
      update();
      if (element === null) return;
      element.addEventListener("scroll", update, { passive: true });
      const Observer = element.ownerDocument.defaultView?.ResizeObserver;
      const observer = Observer === undefined ? null : new Observer(update);
      observer?.observe(element);
      onWatcherCleanup(() => {
        element.removeEventListener("scroll", update);
        observer?.disconnect();
      });
    },
    { flush: "post", immediate: true },
  );

  onScopeDispose(stop);

  return Object.freeze({ scrollOnce, start, stop, update, visible });
}
