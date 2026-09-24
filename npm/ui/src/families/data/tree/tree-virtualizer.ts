import { computed, onScopeDispose, shallowRef } from "vue";

import { useVirtualizer } from "../../interaction/virtualizer/virtualizer.ts";
import type { VirtualizerOptions } from "../../interaction/virtualizer/virtualizer.ts";
import type {
  TreeVirtualizer,
  TreeVirtualizerOptions,
  TreeVirtualizerSource,
} from "./tree-types.ts";

const defaultInitialRect = Object.freeze({ width: 0, height: 320 });

/**
 * Create a windowing adapter for one TreeRoot.
 *
 * Pass the result to `TreeRoot`'s `virtualizer` prop: the tree element becomes
 * the scroll viewport, the root slot's `items` shrink to the rendered window,
 * and keyboard focus scrolls rows into the window before focusing them. Style
 * each row from `getItemStart(item.index)` and size an inner spacer from
 * `totalSize`. Must be called during component setup; it is disposed with the
 * calling scope.
 *
 * @example
 * ```ts
 * const virtualizer = useTreeVirtualizer({ itemSize: 28 });
 * ```
 */
export function useTreeVirtualizer(options: TreeVirtualizerOptions = {}): TreeVirtualizer {
  const source = shallowRef<TreeVirtualizerSource | null>(null);
  const sizing: Pick<VirtualizerOptions, "estimateItemSize" | "itemSize"> =
    options.itemSize === undefined
      ? { estimateItemSize: options.estimateItemSize ?? 32 }
      : { itemSize: options.itemSize };
  const controller = useVirtualizer({
    ...sizing,
    count: () => source.value?.count() ?? 0,
    overscan: options.overscan ?? 4,
    initialRect: options.initialRect ?? defaultInitialRect,
    getItemKey: (index) => source.value?.getKey(index) ?? index,
  });
  let disposed = false;
  onScopeDispose(() => {
    disposed = true;
  });
  const starts = computed(
    () => new Map(controller.virtualItems.value.map((item) => [item.index, item.start])),
  );

  return Object.freeze({
    virtualItems: controller.virtualItems,
    totalSize: controller.totalSize,
    getItemStart: (index: number) => starts.value.get(index) ?? 0,
    connect: (next: TreeVirtualizerSource) => {
      source.value = next;
    },
    // A consumer-owned adapter can be disposed before the tree that uses it unmounts.
    setViewport: (element: Element | null) => {
      if (!disposed) controller.setViewport(element);
    },
    measureElement: (element: Element | null, index: number) => {
      if (!disposed && options.itemSize === undefined) controller.measureElement(element, index);
    },
    scrollToIndex: (index: number) => {
      if (disposed || index < 0 || index >= (source.value?.count() ?? 0)) return;
      controller.scrollToIndex(index, "auto");
    },
  });
}
