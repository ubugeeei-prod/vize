import { computed, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Document-like capability observed by {@link useTextSelection}. */
export interface TextSelectionHost extends EventTarget {
  /** Current selection. */
  getSelection(): Selection | null;
}

/** Options for {@link useTextSelection}. */
export interface UseTextSelectionOptions {
  /**
   * Reactive document capability for alternate runtimes and tests.
   *
   * @default globalThis.document when available
   */
  readonly host?: MaybeRefOrGetter<TextSelectionHost | null | undefined>;
}

/** Reactive selection state returned by {@link useTextSelection}. */
export interface TextSelectionControls {
  /** Current `Selection` object; `null` during server rendering. */
  readonly selection: Readonly<ShallowRef<Selection | null>>;
  /** Selected text. Empty during server rendering. */
  readonly text: ComputedRef<string>;
  /** Selected ranges. */
  readonly ranges: ComputedRef<readonly Range[]>;
  /** Client rectangles of each selected range. */
  readonly rects: ComputedRef<readonly DOMRect[]>;
}

/**
 * Track the document text selection.
 *
 * Re-reads the selection on every `selectionchange`. Because the platform
 * mutates the same `Selection` object in place, a revision counter is used to
 * invalidate the derived `text`, `ranges`, and `rects`. Server renders expose
 * an empty selection; the listener is removed with the owning scope.
 *
 * @param options Document capability.
 * @default options {}
 * @returns Reactive selection, text, ranges, and rectangles.
 */
export function useTextSelection(options: UseTextSelectionOptions = {}): TextSelectionControls {
  const selection = shallowRef<Selection | null>(null);
  const revision = shallowRef(0);

  const stop = watch(
    () => (options.host === undefined ? browserSelectionHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      if (!host) {
        selection.value = null;
        return;
      }
      const update = (): void => {
        selection.value = host.getSelection();
        revision.value += 1;
      };
      update();
      host.addEventListener("selectionchange", update, { passive: true });
      onCleanup(() => host.removeEventListener("selectionchange", update));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  const ranges = computed<readonly Range[]>(() => {
    void revision.value;
    const current = selection.value;
    if (!current) return [];
    const list: Range[] = [];
    for (let index = 0; index < current.rangeCount; index += 1) {
      list.push(current.getRangeAt(index));
    }
    return list;
  });

  return {
    selection,
    text: computed(() => {
      void revision.value;
      return selection.value?.toString() ?? "";
    }),
    ranges,
    rects: computed(() => ranges.value.map((range) => range.getBoundingClientRect())),
  };
}

function browserSelectionHost(): TextSelectionHost | undefined {
  return typeof document !== "undefined" ? document : undefined;
}
