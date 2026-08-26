/**
 * DOM binding between one virtualizer and its scrollable viewport: scroll
 * offset reads and writes plus SSR-safe viewport size observation.
 */

import { createSizeObserver } from "./measure-runtime.ts";
import type { VirtualizerOrientation, VirtualizerRect } from "./virtualizer-types.ts";

/** Callbacks a viewport binding reports into. */
export interface ViewportBindingConfig {
  readonly getOrientation: () => VirtualizerOrientation;
  readonly onScrollOffset: (offset: number) => void;
  readonly onViewportSize: (rect: VirtualizerRect) => void;
}

/** Imperative viewport attachment owned by one virtualizer. */
export interface ViewportBinding {
  readonly element: () => Element | null;
  readonly attach: (element: Element | null) => void;
  readonly applyOffset: (offset: number) => void;
  readonly dispose: () => void;
}

function readOffset(element: Element, orientation: VirtualizerOrientation): number {
  return orientation === "vertical" ? element.scrollTop : element.scrollLeft;
}

/** Create one viewport binding. Attachment is a no-op contract during SSR. */
export function createViewportBinding(config: ViewportBindingConfig): ViewportBinding {
  let element: Element | null = null;
  let releaseScroll: (() => void) | null = null;

  const sizeObserver = createSizeObserver({
    onResize(entries) {
      for (const entry of entries) {
        if (entry.target !== element) continue;
        config.onViewportSize(Object.freeze({ width: entry.width, height: entry.height }));
      }
    },
  });

  const detach = (): void => {
    releaseScroll?.();
    releaseScroll = null;
    if (element) sizeObserver.unobserve(element);
    element = null;
  };

  return Object.freeze<ViewportBinding>({
    element: () => element,
    attach(next) {
      if (next === element) return;
      detach();
      if (!next) return;
      element = next;

      const onScroll = (): void => {
        if (element) config.onScrollOffset(readOffset(element, config.getOrientation()));
      };
      next.addEventListener("scroll", onScroll, { passive: true });
      releaseScroll = () => next.removeEventListener("scroll", onScroll);
      sizeObserver.observe(next);

      if (next.clientWidth > 0 || next.clientHeight > 0) {
        config.onViewportSize(
          Object.freeze({ width: next.clientWidth, height: next.clientHeight }),
        );
      }
      config.onScrollOffset(readOffset(next, config.getOrientation()));
    },
    applyOffset(offset) {
      if (!element) return;
      if (typeof element.scrollTo === "function") {
        element.scrollTo(
          config.getOrientation() === "vertical" ? { top: offset } : { left: offset },
        );
      } else if (config.getOrientation() === "vertical") {
        element.scrollTop = offset;
      } else {
        element.scrollLeft = offset;
      }
    },
    dispose() {
      detach();
      sizeObserver.dispose();
    },
  });
}
