import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue, watch } from "vue";

import type {
  ScrollSpyChangeReason,
  ScrollSpyController,
  ScrollSpyOptions,
} from "./scroll-spy-types.ts";

const setupDiagnostic = "VIZE_UI_SCROLL_SPY_SETUP";
const disposedDiagnostic = "VIZE_UI_SCROLL_SPY_DISPOSED";

function sameIds(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((id, index) => id === right[index]);
}

/**
 * Create an SSR-safe scroll spy for in-page targets such as headings.
 *
 * The active target is the last one whose top edge has crossed the activation
 * line (`offset` pixels below the container top). When the container is
 * scrolled to its end the last visible target wins, so short final sections
 * still activate. Measurements are batched per animation frame and never run
 * on the server; `initialActiveId` is the server and hydration value.
 */
export function createScrollSpy(options: ScrollSpyOptions): ScrollSpyController {
  const activeId = shallowRef<string | null>(options.initialActiveId ?? null);
  const visibleIds = shallowRef<readonly string[]>(Object.freeze([]));
  const view = typeof window === "undefined" ? null : window;
  let frame: number | null = null;
  let disposed = false;
  let unbind: (() => void) | null = null;

  const setActive = (next: string | null, reason: ScrollSpyChangeReason) => {
    const previous = activeId.value;
    if (next === previous) return;
    activeId.value = next;
    options.onActiveChange?.(next, previous, reason);
  };

  const measure = () => {
    frame = null;
    if (disposed || view === null || toValue(options.isDisabled) === true) return;
    const doc = view.document;
    const root = toValue(options.root) ?? null;
    const rootRect = root?.getBoundingClientRect() ?? null;
    const top = rootRect?.top ?? 0;
    const bottom = rootRect?.bottom ?? view.innerHeight;
    const line = top + (toValue(options.offset) ?? 0);
    let candidate: string | null = null;
    const visible: string[] = [];
    for (const id of toValue(options.ids)) {
      const target = doc.getElementById(id);
      if (target === null) continue;
      const rect = target.getBoundingClientRect();
      if (rect.top <= line + 1) candidate = id;
      if (rect.bottom > top && rect.top < bottom) visible.push(id);
    }
    const scroller = root ?? doc.scrollingElement;
    const atEnd =
      scroller !== null &&
      scroller.scrollHeight > scroller.clientHeight &&
      scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 1;
    if (atEnd && visible.length > 0) candidate = visible.at(-1) ?? candidate;
    if (!sameIds(visible, visibleIds.value)) visibleIds.value = Object.freeze(visible);
    setActive(candidate, "scroll");
  };

  const schedule = () => {
    if (disposed || view === null || frame !== null) return;
    frame =
      typeof view.requestAnimationFrame === "function"
        ? view.requestAnimationFrame(measure)
        : view.setTimeout(measure, 16);
  };

  const cancelFrame = () => {
    if (frame === null || view === null) return;
    if (typeof view.cancelAnimationFrame === "function") view.cancelAnimationFrame(frame);
    else view.clearTimeout(frame);
    frame = null;
  };

  const bind = (root: Element | null) => {
    unbind?.();
    if (view === null) return;
    const target: EventTarget = root ?? view;
    target.addEventListener("scroll", schedule, { passive: true });
    view.addEventListener("resize", schedule, { passive: true });
    unbind = () => {
      target.removeEventListener("scroll", schedule);
      view.removeEventListener("resize", schedule);
      unbind = null;
    };
  };

  const stopWatch =
    view === null
      ? () => undefined
      : watch(
          [
            () => toValue(options.root) ?? null,
            () => toValue(options.ids),
            () => toValue(options.offset),
          ],
          ([root], previous) => {
            if (previous === undefined || previous[0] !== root) bind(root);
            schedule();
          },
          { immediate: true, flush: "post" },
        );

  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  return Object.freeze({
    activeId: shallowReadonly(activeId),
    visibleIds: shallowReadonly(visibleIds),
    refresh: () => {
      assertActive();
      cancelFrame();
      measure();
    },
    scrollTo: (id: string, scrollOptions?: ScrollIntoViewOptions) => {
      assertActive();
      const target = view?.document.getElementById(id) ?? null;
      if (target === null) return false;
      target.scrollIntoView(scrollOptions ?? { block: "start" });
      setActive(id, "navigation");
      return true;
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      stopWatch();
      cancelFrame();
      unbind?.();
    },
  });
}

/** Create a scroll spy disposed with the current Vue effect scope. */
export function useScrollSpy(options: ScrollSpyOptions): ScrollSpyController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createScrollSpy(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  ScrollSpyChangeReason,
  ScrollSpyController,
  ScrollSpyOptions,
} from "./scroll-spy-types.ts";
