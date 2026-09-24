import { ref, watch } from "vue";
import type { Ref } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { useResizeObserver } from "./resize-observer.ts";
import type { ResizeObserverHost } from "./resize-observer.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { MaybeRefOrGetter } from "vue";

/** Style property that receives the measured height. */
export type TextareaAutosizeStyleProperty = "height" | "min-height";

/** Options for {@link useTextareaAutosize}. */
export interface UseTextareaAutosizeOptions {
  /**
   * Text bound to the textarea. Changes trigger a resize. A fresh ref is
   * created when omitted.
   *
   * @default ref("")
   */
  readonly input?: Ref<string>;

  /**
   * Style property written with the measured height.
   *
   * @default "height"
   */
  readonly styleProp?: TextareaAutosizeStyleProperty;

  /**
   * Called after each resize with the new height.
   *
   * @default undefined
   */
  readonly onResize?: (height: number) => void;

  /**
   * Resize-observer capability used to re-measure when the width changes.
   *
   * @default globalThis when it provides `ResizeObserver`
   */
  readonly host?: MaybeRefOrGetter<ResizeObserverHost | null | undefined>;
}

/** Textarea-like element measured by {@link useTextareaAutosize}. */
interface AutosizeElement extends Element {
  readonly scrollHeight: number;
  readonly style: Pick<CSSStyleDeclaration, "setProperty" | "removeProperty">;
}

/** State returned by {@link useTextareaAutosize}. */
export interface TextareaAutosizeControls {
  /** Text bound to the textarea (use with `v-model`). */
  readonly input: Ref<string>;
  /** Re-measure immediately. */
  readonly triggerResize: () => void;
}

/**
 * Grow a textarea to fit its content.
 *
 * On each input change (and when the element width changes), the style
 * property is reset to `auto`, `scrollHeight` is read, and the height is
 * written back in pixels. Nothing is measured during server rendering, so the
 * markup keeps its declared `rows`.
 *
 * @param target Reactive textarea target.
 * @param options Bound input, style property, callback, and observer capability.
 * @default options {}
 * @returns Bound input and a manual resize trigger.
 */
export function useTextareaAutosize(
  target: MaybeElementTarget,
  options: UseTextareaAutosizeOptions = {},
): TextareaAutosizeControls {
  const input = options.input ?? ref("");
  const styleProp = options.styleProp ?? "height";
  let lastWidth = -1;

  const triggerResize = (): void => {
    const element = resolveElement(target);
    if (!element || !isAutosizeElement(element)) return;
    element.style.setProperty(styleProp, "auto");
    const height = element.scrollHeight;
    element.style.setProperty(styleProp, `${height}px`);
    options.onResize?.(height);
  };

  const stop = watch([input, () => resolveElement(target)], triggerResize, {
    immediate: true,
    flush: "post",
  });
  const observer = useResizeObserver(
    target,
    (entries) => {
      const width = entries.at(-1)?.contentRect.width ?? -1;
      if (width === lastWidth) return;
      lastWidth = width;
      triggerResize();
    },
    options.host === undefined ? {} : { host: options.host },
  );
  tryOnScopeDispose(() => {
    stop.stop();
    observer.stop();
  });

  return { input, triggerResize };
}

function isAutosizeElement(element: Element): element is AutosizeElement {
  return "style" in element && typeof element.scrollHeight === "number";
}
